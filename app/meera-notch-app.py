#!/usr/bin/env python3
"""
meera-notch Standalone Desktop Application & Top-Bezel Overlay (XWayland / GTK3 + Cairo)
Connects to the Rust daemon `io.github.rizqinrr.MeeraNotch` over session D-Bus.
"""

import os
import sys
import math
import subprocess

# Force X11/XWayland backend so the top-bezel dock window can position itself at (center, y=0)
# and stay above windows on GNOME Wayland (Zorin OS 18.1).
os.environ["GDK_BACKEND"] = "x11"
if "XAUTHORITY" not in os.environ or not os.path.exists(os.environ.get("XAUTHORITY", "")):
    import glob
    auth_files = glob.glob(f"/run/user/{os.getuid()}/.mutter-Xwaylandauth.*")
    if auth_files:
        os.environ["XAUTHORITY"] = auth_files[0]

import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
from gi.repository import Gtk, Gdk, GLib, Gio, Pango
import cairo

BUS_NAME = "io.github.rizqinrr.MeeraNotch"
OBJECT_PATH = "/io/github/rizqinrr/MeeraNotch"
IFACE_NAME = "io.github.rizqinrr.MeeraNotch1"

APP_CSS = b"""
window.meera-control-win {
    background-color: #0A0A0C;
    color: #F4F4F6;
}
.meera-header {
    font-size: 16px;
    font-weight: 700;
    color: #F4F4F6;
}
.meera-sub {
    font-size: 12px;
    color: #A0A4AE;
}
.meera-pill-text {
    font-size: 12.5px;
    font-weight: 600;
    color: #F4F4F6;
}
.meera-pill-accent {
    font-size: 12px;
    font-weight: 700;
    color: #2EC27E;
}
.meera-pill-muted {
    font-size: 12px;
    font-weight: 500;
    color: #A0A4AE;
}
.meera-countdown {
    font-size: 22px;
    font-weight: 700;
    color: #2EC27E;
}
.meera-time-large {
    font-size: 16px;
    font-weight: 700;
    color: #F4F4F6;
}
.meera-card {
    background-color: #16171B;
    border-radius: 10px;
    padding: 10px 12px;
    border: 1px solid #22242C;
}
.meera-cell {
    background-color: #16171B;
    border-radius: 8px;
    padding: 6px 4px;
}
.meera-cell-active {
    background-color: #122B20;
    border: 1px solid #2EC27E;
    border-radius: 8px;
    padding: 6px 4px;
}
button.meera-tab {
    background-image: none;
    background-color: #16171B;
    color: #A0A4AE;
    border: 1px solid #22242C;
    border-radius: 6px;
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 600;
    box-shadow: none;
}
button.meera-tab:hover {
    background-color: #22242C;
    color: #F4F4F6;
}
button.meera-tab-active {
    background-image: none;
    background-color: #122B20;
    color: #2EC27E;
    border: 1px solid #2EC27E;
    border-radius: 6px;
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 700;
    box-shadow: none;
}
button.meera-btn {
    background-image: none;
    background-color: #282A32;
    color: #F4F4F6;
    border: 1px solid #363945;
    border-radius: 6px;
    padding: 5px 10px;
    font-size: 12px;
    font-weight: 600;
    box-shadow: none;
}
button.meera-btn:hover {
    background-color: #363945;
    border-color: #F4F4F6;
}
button.meera-btn-primary {
    background-image: none;
    background-color: #2EC27E;
    color: #0A0A0C;
    border: 1px solid #2EC27E;
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 12.5px;
    font-weight: 700;
    box-shadow: none;
}
button.meera-btn-primary:hover {
    background-color: #43D48E;
}
button.meera-btn-danger {
    background-image: none;
    background-color: #1E1214;
    color: #FF5C5C;
    border: 1px solid #FF5C5C;
    border-radius: 6px;
    padding: 5px 10px;
    font-size: 12px;
    font-weight: 700;
    box-shadow: none;
}
entry.meera-entry {
    background-image: none;
    background-color: #0A0A0C;
    color: #F4F4F6;
    border: 1px solid #282A32;
    border-radius: 6px;
    padding: 4px 8px;
    font-size: 12px;
}
"""


class DaemonClient:
    """Thin D-Bus client wrapper for io.github.rizqinrr.MeeraNotch1."""

    def __init__(self):
        self.bus = None
        self._connect()

    def _connect(self):
        try:
            subprocess.run(
                ["systemctl", "--user", "start", "meera-notchd.service"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=3,
            )
        except Exception:
            pass
        try:
            self.bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        except Exception:
            self.bus = None

    def get_all_props(self):
        if not self.bus:
            self._connect()
        if not self.bus:
            return {}
        try:
            res = self.bus.call_sync(
                BUS_NAME,
                OBJECT_PATH,
                "org.freedesktop.DBus.Properties",
                "GetAll",
                GLib.Variant("(s)", (IFACE_NAME,)),
                GLib.VariantType("(a{sv})"),
                Gio.DBusCallFlags.NONE,
                2000,
                None,
            )
            return res.unpack()[0]
        except Exception:
            return {}

    def get_prop(self, name):
        props = self.get_all_props()
        return props.get(name, {})

    def call(self, method, variant_args=None):
        if not self.bus:
            self._connect()
        if not self.bus:
            return None
        try:
            res = self.bus.call_sync(
                BUS_NAME,
                OBJECT_PATH,
                IFACE_NAME,
                method,
                variant_args,
                None,
                Gio.DBusCallFlags.NONE,
                3000,
                None,
            )
            return res.unpack() if res else None
        except Exception:
            return None


class NotchOverlayWindow(Gtk.Window):
    """Top-center bezel Notch window with Cairo 14px inverse top-corner ears."""

    EAR_RADIUS = 14

    def __init__(self, client, on_open_settings):
        super().__init__(type=Gtk.WindowType.TOPLEVEL)
        self.client = client
        self.on_open_settings = on_open_settings
        self.expanded = False
        self.is_hovered = False
        self.active_tab = "prayer"
        self.open_mode = "click"
        self.armed_action = None
        self.armed_timer = 0

        self.set_title("meera-notch-overlay")
        self.set_decorated(False)
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)
        self.set_keep_above(True)
        self.stick()
        self.set_accept_focus(True)
        self.set_app_paintable(True)
        self.set_resizable(False)

        screen = self.get_screen()
        visual = screen.get_rgba_visual()
        if visual and screen.is_composited():
            self.set_visual(visual)

        self.add_events(
            Gdk.EventMask.ENTER_NOTIFY_MASK
            | Gdk.EventMask.LEAVE_NOTIFY_MASK
            | Gdk.EventMask.BUTTON_PRESS_MASK
        )
        self.connect("draw", self._on_draw)
        self.connect("enter-notify-event", self._on_enter)
        self.connect("leave-notify-event", self._on_leave)
        self.connect("size-allocate", lambda *_: self._reposition_top_center())

        # Root container with 14px horizontal padding for the inverse top-corner ears
        self.outer_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.outer_box.set_margin_start(self.EAR_RADIUS + 14)
        self.outer_box.set_margin_end(self.EAR_RADIUS + 14)
        self.outer_box.set_margin_top(6)
        self.outer_box.set_margin_bottom(8)
        self.add(self.outer_box)

        # Compact Pill Row
        self.pill_event = Gtk.EventBox()
        self.pill_event.set_visible_window(False)
        self.pill_event.connect("button-press-event", self._on_pill_clicked)

        self.pill_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.pill_box.set_halign(Gtk.Align.CENTER)

        self.dot_label = Gtk.Label(label="●")
        self.dot_label.get_style_context().add_class("meera-pill-accent")
        self.prayer_label = Gtk.Label(label="Dzuhur 11:37")
        self.prayer_label.get_style_context().add_class("meera-pill-text")
        self.countdown_label = Gtk.Label(label="6j 2m")
        self.countdown_label.get_style_context().add_class("meera-pill-accent")
        self.sep1 = Gtk.Label(label="·")
        self.sep1.get_style_context().add_class("meera-pill-muted")
        self.pomo_label = Gtk.Label(label="25:00")
        self.pomo_label.get_style_context().add_class("meera-pill-muted")
        self.sep2 = Gtk.Label(label="·")
        self.sep2.get_style_context().add_class("meera-pill-muted")
        self.bat_label = Gtk.Label(label="80%")
        self.bat_label.get_style_context().add_class("meera-pill-muted")

        for w in (
            self.dot_label,
            self.prayer_label,
            self.countdown_label,
            self.sep1,
            self.pomo_label,
            self.sep2,
            self.bat_label,
        ):
            self.pill_box.pack_start(w, False, False, 0)

        self.pill_event.add(self.pill_box)
        self.outer_box.pack_start(self.pill_event, False, False, 0)

        # Expanded Hub Container
        self.hub_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.hub_box.set_margin_top(10)
        self.hub_box.set_size_request(430, -1)

        # Tab bar
        tab_bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        self.tab_buttons = {}
        for tab_id, title in (
            ("prayer", "Sholat & Tilawah"),
            ("system", "Sistem & Cuaca"),
            ("focus", "Fokus & Daya"),
        ):
            btn = Gtk.Button(label=title)
            btn.connect("clicked", self._on_tab_clicked, tab_id)
            tab_bar.pack_start(btn, False, False, 0)
            self.tab_buttons[tab_id] = btn

        gear_btn = Gtk.Button(label="Pengaturan")
        gear_btn.get_style_context().add_class("meera-btn")
        gear_btn.connect("clicked", lambda *_: self.on_open_settings())
        tab_bar.pack_end(gear_btn, False, False, 0)

        self.hub_box.pack_start(tab_bar, False, False, 0)

        self.content_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.hub_box.pack_start(self.content_box, False, False, 0)
        self.outer_box.pack_start(self.hub_box, False, False, 0)

        self._update_tab_styles()
        self.refresh_all()
        GLib.timeout_add_seconds(1, self._tick)

    def _tick(self):
        if self.get_visible():
            self.refresh_pill()
            if self.expanded:
                self.render_active_tab()
            self.set_keep_above(True)
        return True

    def _reposition_top_center(self):
        display = Gdk.Display.get_default()
        if not display:
            return
        mon = display.get_primary_monitor() or display.get_monitor(0)
        geom = mon.get_geometry()
        alloc = self.get_allocation()
        x = geom.x + (geom.width - alloc.width) // 2
        y = geom.y
        self.move(max(0, x), y)

    def _on_draw(self, widget, cr):
        alloc = widget.get_allocation()
        w = alloc.width
        h = alloc.height
        ear = self.EAR_RADIUS
        r = 20 if self.expanded else 16

        # Clear to transparent
        cr.set_operator(cairo.OPERATOR_SOURCE)
        cr.set_source_rgba(0, 0, 0, 0)
        cr.paint()
        cr.set_operator(cairo.OPERATOR_OVER)

        # Draw unified top-bezel notch with left & right inverse ears at y=0
        cr.new_path()
        cr.move_to(0, 0)
        # Left inverse ear: center at (0, ear), clockwise arc from -pi/2 (0,0) to 0 (ear, ear)
        cr.arc(0, ear, ear, -math.pi / 2.0, 0.0)
        # Left vertical edge down to bottom-left corner
        cr.line_to(ear, h - r)
        # Bottom-left convex corner: counter-clockwise arc from pi (ear, h-r) to pi/2 (ear+r, h)
        cr.arc_negative(ear + r, h - r, r, math.pi, math.pi / 2.0)
        # Bottom edge to bottom-right corner
        cr.line_to(w - ear - r, h)
        # Bottom-right convex corner: counter-clockwise arc from pi/2 (w-ear-r, h) to 0 (w-ear, h-r)
        cr.arc_negative(w - ear - r, h - r, r, math.pi / 2.0, 0.0)
        # Right vertical edge up to right inverse ear
        cr.line_to(w - ear, ear)
        # Right inverse ear: center at (w, ear), clockwise arc from pi (w-ear, ear) to 1.5*pi (w, 0)
        cr.arc(w, ear, ear, math.pi, 1.5 * math.pi)
        cr.close_path()

        # Fill: slightly elevated #12141A when hovered in compact pill mode, otherwise #0A0A0C
        if self.is_hovered and not self.expanded:
            cr.set_source_rgba(18 / 255.0, 20 / 255.0, 26 / 255.0, 0.99)
        else:
            cr.set_source_rgba(10 / 255.0, 10 / 255.0, 12 / 255.0, 0.98)
        cr.fill_preserve()

        # Border: emerald highlight #2EC27E on hover, subtle #282A32 otherwise
        if self.is_hovered and not self.expanded:
            cr.set_source_rgba(46 / 255.0, 194 / 255.0, 126 / 255.0, 0.75)
            cr.set_line_width(1.5)
        else:
            cr.set_source_rgba(40 / 255.0, 42 / 255.0, 50 / 255.0, 1.0)
            cr.set_line_width(1.0)
        cr.stroke()
        return False

    def _on_pill_clicked(self, _widget, _event):
        self.set_expanded(not self.expanded)
        return True

    def _on_enter(self, widget, event):
        if event.detail == Gdk.NotifyType.INFERIOR:
            return False
        self.is_hovered = True
        gdk_win = widget.get_window()
        if gdk_win:
            gdk_win.set_cursor(Gdk.Cursor.new_from_name(widget.get_display(), "pointer"))
        if not self.expanded:
            self.outer_box.set_margin_start(self.EAR_RADIUS + 20)
            self.outer_box.set_margin_end(self.EAR_RADIUS + 20)
            self.outer_box.set_margin_bottom(10)
            self.queue_draw()
            GLib.idle_add(self._reposition_top_center)
        return False

    def _on_leave(self, widget, event):
        if event.detail == Gdk.NotifyType.INFERIOR:
            return False
        self.is_hovered = False
        gdk_win = widget.get_window()
        if gdk_win:
            gdk_win.set_cursor(None)
        self.outer_box.set_margin_start(self.EAR_RADIUS + 14)
        self.outer_box.set_margin_end(self.EAR_RADIUS + 14)
        self.outer_box.set_margin_bottom(8)
        self.queue_draw()
        GLib.idle_add(self._reposition_top_center)
        return False

    def set_expanded(self, expanded):
        if self.open_mode == "always":
            expanded = True
        self.expanded = expanded
        self.client.call("SetExpanded", GLib.Variant("(b)", (expanded,)))
        if expanded:
            self.render_active_tab()
            self.hub_box.show_all()
        else:
            self.hub_box.hide()
        self.resize(1, 1)
        GLib.idle_add(self._reposition_top_center)
        self.queue_draw()

    def _on_tab_clicked(self, _btn, tab_id):
        self.active_tab = tab_id
        self._update_tab_styles()
        self.render_active_tab()

    def _update_tab_styles(self):
        for tid, btn in self.tab_buttons.items():
            ctx = btn.get_style_context()
            ctx.remove_class("meera-tab")
            ctx.remove_class("meera-tab-active")
            ctx.add_class("meera-tab-active" if tid == self.active_tab else "meera-tab")

    def refresh_all(self):
        self.refresh_pill()
        if self.expanded:
            self.hub_box.show_all()
            self.render_active_tab()
        else:
            self.hub_box.hide()
        GLib.idle_add(self._reposition_top_center)

    def refresh_pill(self):
        pill = self.client.get_prop("Pill")
        if not pill:
            self.prayer_label.set_text("Service meera-notchd berhenti")
            self.countdown_label.set_text("")
            return
        mode = pill.get("open_mode", "hover")
        if mode != self.open_mode:
            self.open_mode = mode
            if mode == "always" and not self.expanded:
                self.set_expanded(True)

        if pill.get("is_playing") == "true":
            self.prayer_label.set_text(
                f"{pill.get('quran_elapsed', '00:00')} · {pill.get('surah_title', 'Al-Mulk')}"
            )
            self.countdown_label.set_text("Tilawah Aktif")
        else:
            pname = pill.get("prayer_name", "Dzuhur")
            ptime = pill.get("prayer_time", "11:37")
            self.prayer_label.set_text(f"{pname} {ptime}")
            self.countdown_label.set_text(pill.get("countdown", ""))

        self.pomo_label.set_text(pill.get("pomodoro", "25:00"))
        self.bat_label.set_text(pill.get("battery", "80%"))
        GLib.idle_add(self._reposition_top_center)

    def render_active_tab(self):
        for child in self.content_box.get_children():
            self.content_box.remove(child)

        if self.active_tab == "prayer":
            p = self.client.get_prop("Prayer")
            q = self.client.get_prop("Quran")

            hero = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            hero.get_style_context().add_class("meera-card")

            left = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
            lbl_city = Gtk.Label(
                label=f"Menuju {p.get('next_name', 'Dzuhur')} · {p.get('city', 'Bandung')} ({p.get('coords', '-6.92, 107.62')})",
                xalign=0,
            )
            lbl_city.get_style_context().add_class("meera-pill-muted")
            lbl_cd = Gtk.Label(label=p.get("countdown_hms", "-00:25:00"), xalign=0)
            lbl_cd.get_style_context().add_class("meera-countdown")
            left.pack_start(lbl_city, False, False, 0)
            left.pack_start(lbl_cd, False, False, 0)

            right = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
            lbl_t = Gtk.Label(label=f"{p.get('next_time', '11:37')} WIB", xalign=1)
            lbl_t.get_style_context().add_class("meera-time-large")
            lbl_h = Gtk.Label(label=p.get("hijri_date", "1448 H (perkiraan)"), xalign=1)
            lbl_h.get_style_context().add_class("meera-pill-muted")
            right.pack_start(lbl_t, False, False, 0)
            right.pack_start(lbl_h, False, False, 0)

            hero.pack_start(left, True, True, 0)
            hero.pack_end(right, False, False, 0)
            self.content_box.pack_start(hero, False, False, 0)

            strip = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6, homogeneous=True)
            next_name = p.get("next_name", "Dzuhur")
            for name, key in (
                ("Imsak", "imsak"),
                ("Subuh", "subuh"),
                ("Dzuhur", "dzuhur"),
                ("Ashar", "ashar"),
                ("Maghrib", "maghrib"),
                ("Isya", "isya"),
            ):
                cell = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
                is_next = name == next_name
                cell.get_style_context().add_class(
                    "meera-cell-active" if is_next else "meera-cell"
                )
                l1 = Gtk.Label(label=name)
                l1.get_style_context().add_class(
                    "meera-pill-accent" if is_next else "meera-pill-muted"
                )
                l2 = Gtk.Label(label=p.get(key, "--:--"))
                l2.get_style_context().add_class(
                    "meera-pill-accent" if is_next else "meera-pill-text"
                )
                cell.pack_start(l1, False, False, 0)
                cell.pack_start(l2, False, False, 0)
                strip.pack_start(cell, True, True, 0)
            self.content_box.pack_start(strip, False, False, 0)

            qcard = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
            qcard.get_style_context().add_class("meera-card")
            qt = Gtk.Label(
                label=f"{q.get('title', '67. Al-Mulk')} · {q.get('qari_name', 'Mishary Rashid Alafasy')}",
                xalign=0,
            )
            qt.get_style_context().add_class("meera-pill-text")
            qs = Gtk.Label(
                label=f"{q.get('subtitle', 'Makkiyah · 30 Ayat')} · Waktu: {q.get('elapsed', '00:00')}",
                xalign=0,
            )
            qs.get_style_context().add_class("meera-pill-muted")
            qcard.pack_start(qt, False, False, 0)
            qcard.pack_start(qs, False, False, 0)

            qbtns = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
            b_prev = Gtk.Button(label="Sebelumnya")
            b_prev.get_style_context().add_class("meera-btn")
            b_prev.connect(
                "clicked",
                lambda *_: (self.client.call("QuranPrev"), self.refresh_all()),
            )
            is_playing = q.get("is_playing") == "true"
            is_paused = q.get("is_paused") == "true"
            if is_playing:
                play_label = "Jeda Tilawah"
            elif is_paused:
                play_label = "Lanjutkan Tilawah"
            else:
                play_label = "Putar Tilawah"
            b_play = Gtk.Button(label=play_label)
            b_play.get_style_context().add_class("meera-btn-primary")
            b_play.connect(
                "clicked",
                lambda *_: (self.client.call("QuranToggle"), self.refresh_all()),
            )
            b_next = Gtk.Button(label="Berikutnya")
            b_next.get_style_context().add_class("meera-btn")
            b_next.connect(
                "clicked",
                lambda *_: (self.client.call("QuranNext"), self.refresh_all()),
            )
            b_stop = Gtk.Button(label="Berhenti")
            b_stop.get_style_context().add_class("meera-btn-danger")
            b_stop.connect(
                "clicked",
                lambda *_: (self.client.call("QuranStop"), self.refresh_all()),
            )
            for b in (b_prev, b_play, b_next, b_stop):
                qbtns.pack_start(b, False, False, 0)
            qcard.pack_start(qbtns, False, False, 0)
            self.content_box.pack_start(qcard, False, False, 0)

        elif self.active_tab == "system":
            s = self.client.get_prop("System")
            w = self.client.get_prop("Weather")

            c1 = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
            c1.get_style_context().add_class("meera-card")
            l_cpu = Gtk.Label(
                label=f"Intel Core i5-1135G7 (8 Thread) · {s.get('cpu_pct', '0%')} · {s.get('cpu_temp', '45°C')}",
                xalign=0,
            )
            l_cpu.get_style_context().add_class("meera-pill-text")
            l_thr = Gtk.Label(label=s.get("cpu_threads", ""), xalign=0)
            l_thr.set_ellipsize(Pango.EllipsizeMode.END)
            l_thr.get_style_context().add_class("meera-pill-muted")
            c1.pack_start(l_cpu, False, False, 0)
            c1.pack_start(l_thr, False, False, 0)
            self.content_box.pack_start(c1, False, False, 0)

            c2 = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
            c2.get_style_context().add_class("meera-card")
            l_ram = Gtk.Label(
                label=f"Memori RAM: {s.get('ram_summary', '6.2 / 7.5 GiB')}", xalign=0
            )
            l_ram.get_style_context().add_class("meera-pill-text")
            l_bat = Gtk.Label(
                label=f"Baterai BAT0: {s.get('bat_summary', '80%')} · {s.get('bat_limit', 'Batas: 80%')}",
                xalign=0,
            )
            l_bat.get_style_context().add_class("meera-pill-accent")
            l_net = Gtk.Label(
                label=f"Jaringan ({s.get('net_iface', 'wlo1')}): {s.get('net_rx', '0 KB/s')} · {s.get('net_tx', '0 KB/s')}",
                xalign=0,
            )
            l_net.get_style_context().add_class("meera-pill-muted")
            c2.pack_start(l_ram, False, False, 0)
            c2.pack_start(l_bat, False, False, 0)
            c2.pack_start(l_net, False, False, 0)
            self.content_box.pack_start(c2, False, False, 0)

            c3 = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
            c3.get_style_context().add_class("meera-card")
            l_w1 = Gtk.Label(
                label=f"{w.get('summary', 'Bandung · 26°C')} · {w.get('tomorrow', 'Besok: 21 s/d 28°C')}",
                xalign=0,
            )
            l_w1.get_style_context().add_class("meera-pill-text")
            l_w2 = Gtk.Label(
                label=f"{w.get('details', '')} ({w.get('updated', 'Cache lokal')})",
                xalign=0,
            )
            l_w2.get_style_context().add_class("meera-pill-muted")
            c3.pack_start(l_w1, False, False, 0)
            c3.pack_start(l_w2, False, False, 0)
            self.content_box.pack_start(c3, False, False, 0)

        else:
            pomo = self.client.get_prop("Pomodoro")
            c1 = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
            c1.get_style_context().add_class("meera-card")
            lp1 = Gtk.Label(
                label=f"Fokus Pomodoro: {pomo.get('timer', '25:00')} ({pomo.get('state', 'Siap Fokus')})",
                xalign=0,
            )
            lp1.get_style_context().add_class("meera-pill-text")
            lp2 = Gtk.Label(label=pomo.get("stats", "Hari ini: 0 sesi"), xalign=0)
            lp2.get_style_context().add_class("meera-pill-accent")
            c1.pack_start(lp1, False, False, 0)
            c1.pack_start(lp2, False, False, 0)

            pbtns = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
            state = pomo.get("state", "Siap Fokus")
            if "Berjalan" in state:
                start_label = "Jeda Fokus"
            elif "Dijeda" in state:
                start_label = "Lanjutkan Fokus"
            else:
                start_label = "Mulai Fokus"
            b_start = Gtk.Button(label=start_label)
            b_start.get_style_context().add_class("meera-btn-primary")
            b_start.connect(
                "clicked",
                lambda *_: (self.client.call("PomodoroToggle"), self.refresh_all()),
            )
            b_rst = Gtk.Button(label="Atur Ulang")
            b_rst.get_style_context().add_class("meera-btn")
            b_rst.connect(
                "clicked",
                lambda *_: (self.client.call("PomodoroReset"), self.refresh_all()),
            )
            b_50 = Gtk.Button(label="Preset 50/10")
            b_50.get_style_context().add_class("meera-btn")
            b_50.connect(
                "clicked",
                lambda *_: (
                    self.client.call("PomodoroStart", GLib.Variant("(s)", ("50/10",))),
                    self.refresh_all(),
                ),
            )
            for b in (b_start, b_rst, b_50):
                pbtns.pack_start(b, False, False, 0)
            c1.pack_start(pbtns, False, False, 0)
            self.content_box.pack_start(c1, False, False, 0)

            # Notes card
            c2 = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
            c2.get_style_context().add_class("meera-card")
            ln = Gtk.Label(
                label="Catatan Kilat (~/.local/share/meera-notch/notes.txt)", xalign=0
            )
            ln.get_style_context().add_class("meera-pill-muted")
            c2.pack_start(ln, False, False, 0)

            nrow = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
            entry = Gtk.Entry()
            entry.get_style_context().add_class("meera-entry")
            res_note = self.client.call("NotesGet")
            if res_note:
                entry.set_text(res_note[0].strip())
            b_save = Gtk.Button(label="Simpan")
            b_save.get_style_context().add_class("meera-btn")
            b_save.connect(
                "clicked",
                lambda *_: self.client.call(
                    "NotesSet", GLib.Variant("(s)", (entry.get_text(),))
                ),
            )
            nrow.pack_start(entry, True, True, 0)
            nrow.pack_start(b_save, False, False, 0)
            c2.pack_start(nrow, False, False, 0)
            self.content_box.pack_start(c2, False, False, 0)

            # Power card
            c3 = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
            c3.get_style_context().add_class("meera-card")
            lpw = Gtk.Label(
                label="Aksi Sistem (Mulai Ulang & Matikan butuh 2x klik dalam 3 detik)",
                xalign=0,
            )
            lpw.get_style_context().add_class("meera-pill-muted")
            c3.pack_start(lpw, False, False, 0)

            prow = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
            b_lock = Gtk.Button(label="Kunci Layar")
            b_lock.get_style_context().add_class("meera-btn")
            b_lock.connect(
                "clicked",
                lambda *_: self.client.call("Power", GLib.Variant("(s)", ("lock",))),
            )
            b_sleep = Gtk.Button(label="Tidur")
            b_sleep.get_style_context().add_class("meera-btn")
            b_sleep.connect(
                "clicked",
                lambda *_: self.client.call("Power", GLib.Variant("(s)", ("suspend",))),
            )
            b_reb = Gtk.Button(
                label="Klik Lagi: Ulang"
                if self.armed_action == "reboot"
                else "Mulai Ulang"
            )
            b_reb.get_style_context().add_class("meera-btn")
            b_reb.connect("clicked", lambda *_: self._handle_armed("reboot"))
            b_off = Gtk.Button(
                label="Klik Lagi: Mati"
                if self.armed_action == "poweroff"
                else "Matikan"
            )
            b_off.get_style_context().add_class("meera-btn-danger")
            b_off.connect("clicked", lambda *_: self._handle_armed("poweroff"))

            for b in (b_lock, b_sleep, b_reb, b_off):
                prow.pack_start(b, False, False, 0)
            c3.pack_start(prow, False, False, 0)
            self.content_box.pack_start(c3, False, False, 0)

        self.content_box.show_all()
        GLib.idle_add(self._reposition_top_center)

    def _handle_armed(self, action):
        if self.armed_action == action:
            self.armed_action = None
            self.client.call("Power", GLib.Variant("(s)", (action,)))
            return
        self.armed_action = action
        self.render_active_tab()
        if self.armed_timer:
            GLib.source_remove(self.armed_timer)
        self.armed_timer = GLib.timeout_add_seconds(3, self._disarm)

    def _disarm(self):
        self.armed_timer = 0
        self.armed_action = None
        if self.expanded and self.active_tab == "focus":
            self.render_active_tab()
        return False


class ControlLauncherWindow(Gtk.Window):
    """Main application window opened when clicking the meera-notch icon in the menu."""

    def __init__(self, client, notch_win):
        super().__init__(title="meera-notch")
        self.client = client
        self.notch_win = notch_win
        self.set_default_size(460, 380)
        self.set_position(Gtk.WindowPosition.CENTER)
        self.get_style_context().add_class("meera-control-win")
        self.connect("delete-event", self._on_close_request)

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        root.set_margin_top(18)
        root.set_margin_bottom(18)
        root.set_margin_start(18)
        root.set_margin_end(18)
        self.add(root)

        # Header + Launch/Stop Card
        hero = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        hero.get_style_context().add_class("meera-card")

        title = Gtk.Label(label="meera-notch · Produktivitas & Ibadah", xalign=0)
        title.get_style_context().add_class("meera-header")
        self.status_lbl = Gtk.Label(
            label="Status: Notch aktif menempel di bibir atas layar (y = 0)", xalign=0
        )
        self.status_lbl.get_style_context().add_class("meera-pill-accent")

        btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.toggle_btn = Gtk.Button(label="Sembunyikan / Hentikan Notch")
        self.toggle_btn.get_style_context().add_class("meera-btn-primary")
        self.toggle_btn.connect("clicked", self._on_toggle_notch)

        expand_btn = Gtk.Button(label="Buka / Tutup Hub Atas")
        expand_btn.get_style_context().add_class("meera-btn")
        expand_btn.connect(
            "clicked", lambda *_: self.notch_win.set_expanded(not self.notch_win.expanded)
        )

        btn_row.pack_start(self.toggle_btn, False, False, 0)
        btn_row.pack_start(expand_btn, False, False, 0)

        hero.pack_start(title, False, False, 0)
        hero.pack_start(self.status_lbl, False, False, 0)
        hero.pack_start(btn_row, False, False, 0)
        root.pack_start(hero, False, False, 0)

        # Interaction Mode Card
        mode_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        mode_card.get_style_context().add_class("meera-card")
        m_title = Gtk.Label(label="Cara Membuka Notch di Bibir Atas Layar", xalign=0)
        m_title.get_style_context().add_class("meera-pill-text")
        mode_card.pack_start(m_title, False, False, 0)

        m_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        for mode_id, label in (
            ("hover", "Hover (Sentuh)"),
            ("click", "Klik"),
            ("always", "Selalu Terbuka"),
        ):
            b = Gtk.Button(label=label)
            b.get_style_context().add_class("meera-btn")
            b.connect("clicked", self._on_set_mode, mode_id)
            m_row.pack_start(b, False, False, 0)
        mode_card.pack_start(m_row, False, False, 0)
        root.pack_start(mode_card, False, False, 0)

        # Qari & Location Card
        q_card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        q_card.get_style_context().add_class("meera-card")
        q_title = Gtk.Label(
            label="Pilih Qari Murottal & Lokasi Jadwal Sholat (Kemenag RI)", xalign=0
        )
        q_title.get_style_context().add_class("meera-pill-text")
        q_card.pack_start(q_title, False, False, 0)

        q_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        for q_id, q_label in (
            ("alafasy", "Alafasy"),
            ("sudais", "As-Sudais"),
            ("ghamdi", "Al-Ghamdi"),
            ("husary", "Al-Husary"),
        ):
            b = Gtk.Button(label=q_label)
            b.get_style_context().add_class("meera-btn")
            b.connect(
                "clicked",
                lambda _b, qid=q_id: (
                    self.client.call("QuranPlay", GLib.Variant("(sq)", (qid, 67))),
                    self.notch_win.refresh_all(),
                ),
            )
            q_row.pack_start(b, False, False, 0)
        q_card.pack_start(q_row, False, False, 0)

        loc_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        p = self.client.get_prop("Prayer")
        city = p.get("city", "Bandung")
        coords = p.get("coords", "-6.92, 107.62")
        self.loc_lbl = Gtk.Label(label=f"Lokasi: {city} ({coords})", xalign=0)
        self.loc_lbl.set_ellipsize(Pango.EllipsizeMode.END)
        self.loc_lbl.get_style_context().add_class("meera-pill-muted")
        loc_btn = Gtk.Button(label="Deteksi Ulang IP")
        loc_btn.get_style_context().add_class("meera-btn")
        loc_btn.connect("clicked", self._on_refresh_loc)
        loc_row.pack_start(self.loc_lbl, True, True, 0)
        loc_row.pack_end(loc_btn, False, False, 0)
        q_card.pack_start(loc_row, False, False, 0)
        root.pack_start(q_card, False, False, 0)

        # Quit row
        foot = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        note = Gtk.Label(
            label="Menutup jendela ini tetap membiarkan Notch tampil di atas layar.",
            xalign=0,
        )
        note.get_style_context().add_class("meera-pill-muted")
        quit_btn = Gtk.Button(label="Keluar Aplikasi")
        quit_btn.get_style_context().add_class("meera-btn-danger")
        quit_btn.connect("clicked", lambda *_: Gtk.main_quit())
        foot.pack_start(note, True, True, 0)
        foot.pack_end(quit_btn, False, False, 0)
        root.pack_start(foot, False, False, 0)

        self._sync_button_state()

    def _sync_button_state(self):
        if self.notch_win.get_visible():
            self.status_lbl.set_text(
                "Status: Notch AKTIF menempel di bibir atas layar (y = 0)"
            )
            self.toggle_btn.set_label("Hentikan / Sembunyikan Notch")
        else:
            self.status_lbl.set_text("Status: Notch NONAKTIF (tersembunyi)")
            self.toggle_btn.set_label("Jalankan Notch Sekarang")

    def _on_toggle_notch(self, _btn):
        if self.notch_win.get_visible():
            self.notch_win.hide()
        else:
            self.notch_win.show_all()
            self.notch_win.refresh_all()
        self._sync_button_state()

    def _on_set_mode(self, _btn, mode_id):
        self.client.call("SetOpenMode", GLib.Variant("(s)", (mode_id,)))
        self.notch_win.open_mode = mode_id
        if mode_id == "always":
            self.notch_win.set_expanded(True)
        else:
            self.notch_win.set_expanded(False)

    def _on_refresh_loc(self, btn):
        btn.set_label("Mendeteksi...")
        res = self.client.call("RefreshLocation")
        if res and len(res) > 0 and isinstance(res[0], dict):
            city = res[0].get("city", "Bandung")
            coords = res[0].get("coords", "")
            self.loc_lbl.set_text(f"Lokasi: {city} ({coords})" if coords else f"Lokasi: {city}")
        btn.set_label("Deteksi Ulang IP")
        self.notch_win.refresh_all()

    def _on_close_request(self, _win, _event):
        if self.notch_win.get_visible():
            self.hide()
            return True
        Gtk.main_quit()
        return False


def main():
    provider = Gtk.CssProvider()
    provider.load_from_data(APP_CSS)
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
    )

    client = DaemonClient()
    ctrl_holder = {}

    def open_settings():
        win = ctrl_holder.get("win")
        if win:
            win._sync_button_state()
            win.show_all()
            win.present()

    notch_win = NotchOverlayWindow(client, open_settings)
    notch_win.show_all()
    notch_win.refresh_all()

    ctrl_win = ControlLauncherWindow(client, notch_win)
    ctrl_holder["win"] = ctrl_win

    if "--background" not in sys.argv:
        ctrl_win.show_all()

    Gtk.main()


if __name__ == "__main__":
    main()
