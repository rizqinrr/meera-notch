import Clutter from 'gi://Clutter';
import GObject from 'gi://GObject';
import St from 'gi://St';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';

const DBUS_INTERFACE = `
<node>
  <interface name="io.github.rizqinrr.MeeraNotch1">
    <property name="Version" type="s" access="read"/>
    <property name="Pill" type="a{ss}" access="read"/>
    <property name="Prayer" type="a{ss}" access="read"/>
    <property name="Quran" type="a{ss}" access="read"/>
    <property name="System" type="a{ss}" access="read"/>
    <property name="Weather" type="a{ss}" access="read"/>
    <property name="Pomodoro" type="a{ss}" access="read"/>
    <method name="Ping">
      <arg type="s" direction="out" name="reply"/>
    </method>
    <method name="SetExpanded">
      <arg type="b" direction="in" name="expanded"/>
    </method>
    <method name="QuranPlay">
      <arg type="s" direction="in" name="qari"/>
      <arg type="q" direction="in" name="surah"/>
      <arg type="a{ss}" direction="out" name="state"/>
    </method>
    <method name="QuranToggle">
      <arg type="a{ss}" direction="out" name="state"/>
    </method>
    <method name="QuranStop">
      <arg type="a{ss}" direction="out" name="state"/>
    </method>
    <method name="QuranNext">
      <arg type="a{ss}" direction="out" name="state"/>
    </method>
    <method name="QuranPrev">
      <arg type="a{ss}" direction="out" name="state"/>
    </method>
    <method name="PomodoroStart">
      <arg type="s" direction="in" name="preset"/>
      <arg type="a{ss}" direction="out" name="state"/>
    </method>
    <method name="PomodoroToggle">
      <arg type="a{ss}" direction="out" name="state"/>
    </method>
    <method name="PomodoroReset">
      <arg type="a{ss}" direction="out" name="state"/>
    </method>
    <method name="NotesGet">
      <arg type="s" direction="out" name="content"/>
    </method>
    <method name="NotesSet">
      <arg type="s" direction="in" name="content"/>
      <arg type="b" direction="out" name="ok"/>
    </method>
    <method name="Power">
      <arg type="s" direction="in" name="action"/>
      <arg type="s" direction="out" name="status"/>
    </method>
  </interface>
</node>`;

const MeeraProxy = Gio.DBusProxy.makeProxyWrapper(DBUS_INTERFACE);
const EAR_SIZE = 14;

function createEarDrawingArea(isLeft) {
    const ear = new St.DrawingArea({
        style_class: 'meera-notch-ear',
        width: EAR_SIZE,
        height: EAR_SIZE,
        y_align: Clutter.ActorAlign.START,
    });
    ear.connect('repaint', (area) => {
        const cr = area.get_context();
        cr.setSourceRGBA(10 / 255, 10 / 255, 12 / 255, 1.0);
        if (isLeft) {
            cr.moveTo(0, 0);
            cr.lineTo(EAR_SIZE, 0);
            cr.lineTo(EAR_SIZE, EAR_SIZE);
            cr.curveTo(EAR_SIZE, 4, EAR_SIZE - 4, 0, 0, 0);
        } else {
            cr.moveTo(0, 0);
            cr.lineTo(EAR_SIZE, 0);
            cr.curveTo(4, 0, 0, 4, 0, EAR_SIZE);
            cr.lineTo(0, 0);
        }
        cr.closePath();
        cr.fill();
        cr.$dispose();
    });
    return ear;
}

export const NotchWidget = GObject.registerClass(
class NotchWidget extends St.Widget {
    _init() {
        super._init({
            name: 'meera-notch',
            style_class: 'meera-notch-wrapper',
            reactive: true,
            track_hover: true,
            can_focus: true,
            layout_manager: new Clutter.BoxLayout({
                orientation: Clutter.Orientation.HORIZONTAL,
                spacing: 0,
            }),
        });

        this._expanded = false;
        this._openMode = 'click';
        this._activeTab = 'prayer';
        this._armedAction = null;
        this._armedTimer = 0;

        this._leftEar = createEarDrawingArea(true);
        this._rightEar = createEarDrawingArea(false);

        this._body = new St.BoxLayout({
            style_class: 'meera-notch-body',
            vertical: true,
            reactive: true,
        });

        this._compactRow = new St.BoxLayout({
            vertical: false,
            y_align: Clutter.ActorAlign.CENTER,
            x_align: Clutter.ActorAlign.CENTER,
            style: 'height: 36px; spacing: 10px;',
        });

        this._statusDot = new St.Widget({
            style_class: 'meera-notch-dot',
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._prayerLabel = new St.Label({
            text: 'Ashar 15:12',
            style_class: 'meera-notch-text-primary',
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._countdownLabel = new St.Label({
            text: '25m',
            style_class: 'meera-notch-text-accent',
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._sep1 = new St.Widget({
            style_class: 'meera-notch-sep',
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._pomodoroLabel = new St.Label({
            text: '25:00',
            style_class: 'meera-notch-text-muted',
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._sep2 = new St.Widget({
            style_class: 'meera-notch-sep',
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._batteryLabel = new St.Label({
            text: '80%',
            style_class: 'meera-notch-text-primary',
            y_align: Clutter.ActorAlign.CENTER,
        });

        this._compactRow.add_child(this._statusDot);
        this._compactRow.add_child(this._prayerLabel);
        this._compactRow.add_child(this._countdownLabel);
        this._compactRow.add_child(this._sep1);
        this._compactRow.add_child(this._pomodoroLabel);
        this._compactRow.add_child(this._sep2);
        this._compactRow.add_child(this._batteryLabel);

        this._hubContainer = new St.BoxLayout({
            vertical: true,
            visible: false,
            style: 'spacing: 10px; padding-top: 8px; width: 424px;',
        });
        this._buildHubContent();

        this._body.add_child(this._compactRow);
        this._body.add_child(this._hubContainer);

        this.add_child(this._leftEar);
        this.add_child(this._body);
        this.add_child(this._rightEar);

        this.connect('notify::hover', () => this._onHoverChanged());
        this.connect('button-press-event', () => {
            if (this._openMode === 'click') {
                this._setExpanded(!this._expanded);
                return Clutter.EVENT_STOP;
            }
            return Clutter.EVENT_PROPAGATE;
        });
        this.connect('key-press-event', (_, event) => {
            if (event.get_key_symbol() === Clutter.KEY_Escape && this._expanded && this._openMode !== 'always') {
                this._setExpanded(false);
                return Clutter.EVENT_STOP;
            }
            return Clutter.EVENT_PROPAGATE;
        });

        this._initDBus();
    }

    _buildHubContent() {
        const tabRow = new St.BoxLayout({
            vertical: false,
            style: 'spacing: 8px;',
        });

        this._tabButtons = {};
        const tabs = [
            { id: 'prayer', label: 'Sholat & Tilawah' },
            { id: 'system', label: 'Sistem & Cuaca' },
            { id: 'focus', label: 'Fokus & Catatan' },
        ];

        for (const t of tabs) {
            const btn = new St.Button({
                label: t.label,
                style_class: t.id === this._activeTab ? 'meera-notch-tab-active' : 'meera-notch-tab',
                can_focus: true,
            });
            btn.connect('clicked', () => this._switchTab(t.id));
            tabRow.add_child(btn);
            this._tabButtons[t.id] = btn;
        }

        this._tabContentBox = new St.BoxLayout({
            vertical: true,
            style: 'spacing: 10px;',
        });

        this._hubContainer.add_child(tabRow);
        this._hubContainer.add_child(this._tabContentBox);
        this._renderActiveTab();
    }

    _switchTab(tabId) {
        this._activeTab = tabId;
        for (const [id, btn] of Object.entries(this._tabButtons)) {
            btn.style_class = id === tabId ? 'meera-notch-tab-active' : 'meera-notch-tab';
        }
        this._renderActiveTab();
    }

    _renderActiveTab() {
        this._tabContentBox.destroy_all_children();

        if (this._activeTab === 'prayer') {
            const p = (this._proxy && this._proxy.Prayer) || {};
            const q = (this._proxy && this._proxy.Quran) || {};

            const hero = new St.BoxLayout({
                style_class: 'meera-notch-card',
                vertical: false,
                x_expand: true,
            });
            const leftCol = new St.BoxLayout({ vertical: true, x_expand: true });
            leftCol.add_child(new St.Label({
                text: `Menuju ${p.next_name || 'Ashar'} · ${p.city || 'Bandung'} (${p.coords || '-6.92, 107.62'})`,
                style_class: 'meera-notch-text-muted',
            }));
            leftCol.add_child(new St.Label({
                text: p.countdown_hms || '-00:24:48',
                style_class: 'meera-notch-countdown',
            }));

            const rightCol = new St.BoxLayout({ vertical: true, x_align: Clutter.ActorAlign.END });
            rightCol.add_child(new St.Label({
                text: `${p.next_time || '15:12'} WIB`,
                style_class: 'meera-notch-time-large',
            }));
            rightCol.add_child(new St.Label({
                text: p.hijri_date || '16 Dzulqaidah 1447 H (perkiraan)',
                style_class: 'meera-notch-text-muted',
            }));

            hero.add_child(leftCol);
            hero.add_child(rightCol);
            this._tabContentBox.add_child(hero);

            const strip = new St.BoxLayout({ vertical: false, style: 'spacing: 6px;' });
            const items = [
                ['Imsak', p.imsak || '04:21'],
                ['Subuh', p.subuh || '04:31'],
                ['Dzuhur', p.dzuhur || '11:48'],
                ['Ashar', p.ashar || '15:12'],
                ['Maghrib', p.maghrib || '17:45'],
                ['Isya', p.isya || '18:56'],
            ];
            for (const [name, time] of items) {
                const isNext = name === (p.next_name || 'Ashar');
                const cell = new St.BoxLayout({
                    style_class: isNext ? 'meera-notch-prayer-cell-active' : 'meera-notch-prayer-cell',
                    vertical: true,
                    x_expand: true,
                    x_align: Clutter.ActorAlign.CENTER,
                });
                cell.add_child(new St.Label({
                    text: name,
                    style_class: isNext ? 'meera-notch-text-accent' : 'meera-notch-text-muted',
                    x_align: Clutter.ActorAlign.CENTER,
                }));
                cell.add_child(new St.Label({
                    text: time,
                    style_class: isNext ? 'meera-notch-text-accent' : 'meera-notch-text-primary',
                    x_align: Clutter.ActorAlign.CENTER,
                }));
                strip.add_child(cell);
            }
            this._tabContentBox.add_child(strip);

            const quranCard = new St.BoxLayout({
                style_class: 'meera-notch-card',
                vertical: true,
                style: 'spacing: 8px;',
            });
            quranCard.add_child(new St.Label({
                text: `${q.title || '67. Al-Mulk (Kerajaan)'} · ${q.qari_name || 'Mishary Rashid Alafasy'}`,
                style_class: 'meera-notch-text-primary',
            }));
            quranCard.add_child(new St.Label({
                text: `${q.subtitle || 'Makkiyah · 30 Ayat'} · Waktu: ${q.elapsed || '00:00'}`,
                style_class: 'meera-notch-text-muted',
            }));

            const ctrlRow = new St.BoxLayout({ vertical: false, style: 'spacing: 8px; padding-top: 4px;' });
            const prevBtn = new St.Button({ label: 'Surah Sebelumnya', style_class: 'meera-notch-btn', can_focus: true });
            const playBtn = new St.Button({
                label: q.is_playing === 'true' ? 'Jeda Tilawah' : 'Putar Tilawah',
                style_class: 'meera-notch-btn-primary',
                can_focus: true,
            });
            const nextBtn = new St.Button({ label: 'Surah Berikutnya', style_class: 'meera-notch-btn', can_focus: true });

            prevBtn.connect('clicked', () => {
                if (this._proxy) this._proxy.QuranPrevRemote(() => this._renderActiveTab());
            });
            playBtn.connect('clicked', () => {
                if (this._proxy) this._proxy.QuranToggleRemote(() => {
                    this._updateFromProxy();
                    this._renderActiveTab();
                });
            });
            nextBtn.connect('clicked', () => {
                if (this._proxy) this._proxy.QuranNextRemote(() => this._renderActiveTab());
            });

            ctrlRow.add_child(prevBtn);
            ctrlRow.add_child(playBtn);
            ctrlRow.add_child(nextBtn);
            quranCard.add_child(ctrlRow);

            this._tabContentBox.add_child(quranCard);
        } else if (this._activeTab === 'system') {
            const s = (this._proxy && this._proxy.System) || {};
            const w = (this._proxy && this._proxy.Weather) || {};

            const cpuCard = new St.BoxLayout({
                style_class: 'meera-notch-card',
                vertical: true,
                style: 'spacing: 4px;',
            });
            cpuCard.add_child(new St.Label({
                text: `Intel Core i5-1135G7 (8 Thread) · ${s.cpu_pct || '18%'} · ${s.cpu_temp || '48°C'}`,
                style_class: 'meera-notch-text-primary',
            }));
            cpuCard.add_child(new St.Label({
                text: s.cpu_threads || 'Load: 0.52',
                style_class: 'meera-notch-text-muted',
            }));
            this._tabContentBox.add_child(cpuCard);

            const ramCard = new St.BoxLayout({
                style_class: 'meera-notch-card',
                vertical: true,
                style: 'spacing: 4px;',
            });
            ramCard.add_child(new St.Label({
                text: `Memori RAM: ${s.ram_summary || '6.2 / 7.5 GiB'}`,
                style_class: 'meera-notch-text-primary',
            }));
            ramCard.add_child(new St.Label({
                text: `Baterai BAT0: ${s.bat_summary || '80%'} · ${s.bat_limit || 'Batas aktif: 80%'}`,
                style_class: 'meera-notch-text-accent',
            }));
            ramCard.add_child(new St.Label({
                text: `Jaringan (${s.net_iface || 'wlan0'}): ${s.net_rx || 'Turun 0 KB/s'} · ${s.net_tx || 'Naik 0 KB/s'}`,
                style_class: 'meera-notch-text-muted',
            }));
            this._tabContentBox.add_child(ramCard);

            const wCard = new St.BoxLayout({
                style_class: 'meera-notch-card',
                vertical: true,
                style: 'spacing: 4px;',
            });
            wCard.add_child(new St.Label({
                text: `${w.summary || 'Bandung · 26°C Berawan'} · ${w.tomorrow || 'Besok: 21 s/d 28°C'}`,
                style_class: 'meera-notch-text-primary',
            }));
            wCard.add_child(new St.Label({
                text: `${w.details || 'Kelembapan 78%'} (${w.updated || 'Cache lokal'})`,
                style_class: 'meera-notch-text-muted',
            }));
            this._tabContentBox.add_child(wCard);
        } else {
            const pomo = (this._proxy && this._proxy.Pomodoro) || {};

            const pomoCard = new St.BoxLayout({
                style_class: 'meera-notch-card',
                vertical: true,
                style: 'spacing: 6px;',
            });
            pomoCard.add_child(new St.Label({
                text: `Fokus Pomodoro: ${pomo.timer || '25:00'} (${pomo.state || 'Siap Fokus'})`,
                style_class: 'meera-notch-text-primary',
            }));
            pomoCard.add_child(new St.Label({
                text: pomo.stats || 'Hari ini: 0 sesi',
                style_class: 'meera-notch-text-accent',
            }));

            const pBtns = new St.BoxLayout({ vertical: false, style: 'spacing: 8px;' });
            const startBtn = new St.Button({ label: 'Mulai / Jeda', style_class: 'meera-notch-btn-primary', can_focus: true });
            const resetBtn = new St.Button({ label: 'Atur Ulang', style_class: 'meera-notch-btn', can_focus: true });
            const p50Btn = new St.Button({ label: 'Preset 50/10', style_class: 'meera-notch-btn', can_focus: true });

            startBtn.connect('clicked', () => {
                if (this._proxy) this._proxy.PomodoroToggleRemote(() => {
                    this._updateFromProxy();
                    this._renderActiveTab();
                });
            });
            resetBtn.connect('clicked', () => {
                if (this._proxy) this._proxy.PomodoroResetRemote(() => {
                    this._updateFromProxy();
                    this._renderActiveTab();
                });
            });
            p50Btn.connect('clicked', () => {
                if (this._proxy) this._proxy.PomodoroStartRemote('50/10', () => {
                    this._updateFromProxy();
                    this._renderActiveTab();
                });
            });

            pBtns.add_child(startBtn);
            pBtns.add_child(resetBtn);
            pBtns.add_child(p50Btn);
            pomoCard.add_child(pBtns);
            this._tabContentBox.add_child(pomoCard);

            const notesCard = new St.BoxLayout({
                style_class: 'meera-notch-card',
                vertical: true,
                style: 'spacing: 6px;',
            });
            notesCard.add_child(new St.Label({
                text: 'Catatan Kilat (~/.local/share/meera-notch/notes.txt)',
                style_class: 'meera-notch-text-muted',
            }));
            const noteRow = new St.BoxLayout({ vertical: false, style: 'spacing: 8px;' });
            const noteEntry = new St.Entry({
                style_class: 'meera-notch-entry',
                hint_text: 'Tulis catatan fokus...',
                text: this._cachedNotes || '',
                can_focus: true,
                x_expand: true,
            });
            const saveNoteBtn = new St.Button({
                label: 'Simpan',
                style_class: 'meera-notch-btn',
                can_focus: true,
            });
            if (this._proxy && this._cachedNotes === undefined) {
                this._proxy.NotesGetRemote(([txt]) => {
                    this._cachedNotes = (txt || '').trim();
                    noteEntry.set_text(this._cachedNotes);
                });
            }
            saveNoteBtn.connect('clicked', () => {
                const val = noteEntry.get_text();
                this._cachedNotes = val;
                if (this._proxy) {
                    this._proxy.NotesSetRemote(val, () => {});
                }
            });
            noteRow.add_child(noteEntry);
            noteRow.add_child(saveNoteBtn);
            notesCard.add_child(noteRow);
            this._tabContentBox.add_child(notesCard);

            const pwrCard = new St.BoxLayout({
                style_class: 'meera-notch-card',
                vertical: true,
                style: 'spacing: 6px;',
            });
            pwrCard.add_child(new St.Label({
                text: 'Aksi Sistem (Mulai Ulang & Matikan butuh 2x klik dalam 3 detik)',
                style_class: 'meera-notch-text-muted',
            }));

            const pwrRow = new St.BoxLayout({ vertical: false, style: 'spacing: 8px;' });
            const lockBtn = new St.Button({ label: 'Kunci Layar', style_class: 'meera-notch-btn', can_focus: true });
            const sleepBtn = new St.Button({ label: 'Tidur', style_class: 'meera-notch-btn', can_focus: true });
            const rebootBtn = new St.Button({
                label: this._armedAction === 'reboot' ? 'Klik Lagi: Ulang' : 'Mulai Ulang',
                style_class: 'meera-notch-btn',
                can_focus: true,
            });
            const offBtn = new St.Button({
                label: this._armedAction === 'poweroff' ? 'Klik Lagi: Mati' : 'Matikan',
                style_class: 'meera-notch-btn-danger',
                can_focus: true,
            });

            lockBtn.connect('clicked', () => { if (this._proxy) this._proxy.PowerRemote('lock', () => {}); });
            sleepBtn.connect('clicked', () => { if (this._proxy) this._proxy.PowerRemote('suspend', () => {}); });
            rebootBtn.connect('clicked', () => this._handleArmedPower('reboot'));
            offBtn.connect('clicked', () => this._handleArmedPower('poweroff'));

            pwrRow.add_child(lockBtn);
            pwrRow.add_child(sleepBtn);
            pwrRow.add_child(rebootBtn);
            pwrRow.add_child(offBtn);
            pwrCard.add_child(pwrRow);
            this._tabContentBox.add_child(pwrCard);
        }
    }

    _handleArmedPower(action) {
        if (this._armedAction === action) {
            this._armedAction = null;
            if (this._proxy) this._proxy.PowerRemote(action, () => {});
            return;
        }
        this._armedAction = action;
        this._renderActiveTab();
        if (this._armedTimer) GLib.source_remove(this._armedTimer);
        this._armedTimer = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 3, () => {
            this._armedTimer = 0;
            this._armedAction = null;
            if (this._expanded && this._activeTab === 'focus') this._renderActiveTab();
            return GLib.SOURCE_REMOVE;
        });
    }

    _onHoverChanged() {
        if (this._openMode !== 'hover') return;
        if (this.hover) {
            if (this._collapseTimeout) {
                GLib.source_remove(this._collapseTimeout);
                this._collapseTimeout = 0;
            }
            if (!this._expanded && !this._hoverTimeout) {
                this._hoverTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 180, () => {
                    this._hoverTimeout = 0;
                    this._setExpanded(true);
                    return GLib.SOURCE_REMOVE;
                });
            }
        } else {
            if (this._hoverTimeout) {
                GLib.source_remove(this._hoverTimeout);
                this._hoverTimeout = 0;
            }
            if (this._expanded && !this._collapseTimeout) {
                this._collapseTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 300, () => {
                    this._collapseTimeout = 0;
                    this._setExpanded(false);
                    return GLib.SOURCE_REMOVE;
                });
            }
        }
    }

    _setExpanded(expanded) {
        if (this._expanded === expanded) return;
        this._expanded = expanded;
        this._body.style_class = expanded ? 'meera-notch-body-expanded' : 'meera-notch-body';
        if (expanded) {
            this._hubContainer.opacity = 0;
            this._hubContainer.visible = true;
            this._hubContainer.ease({
                opacity: 255,
                duration: 200,
                mode: Clutter.AnimationMode.EASE_OUT_QUAD,
            });
        } else {
            this._hubContainer.visible = false;
        }
        if (this._proxy) {
            try {
                this._proxy.SetExpandedRemote(expanded, () => {
                    if (expanded) this._renderActiveTab();
                });
            } catch (_) {}
        }
    }

    _initDBus() {
        try {
            MeeraProxy(
                Gio.DBus.session,
                'io.github.rizqinrr.MeeraNotch',
                '/io/github/rizqinrr/MeeraNotch',
                (proxy, error) => {
                    if (error) {
                        this._setOfflineState();
                        return;
                    }
                    this._proxy = proxy;
                    this._updateFromProxy();

                    this._pollId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 5, () => {
                        this._updateFromProxy();
                        if (this._expanded) this._renderActiveTab();
                        return GLib.SOURCE_CONTINUE;
                    });
                }
            );
        } catch (_) {
            this._setOfflineState();
        }
    }

    _setOfflineState() {
        this._statusDot.style_class = 'meera-notch-dot-error';
        this._prayerLabel.text = 'Service meera-notchd berhenti';
        this._countdownLabel.text = '';
    }

    _updateFromProxy() {
        if (!this._proxy) return;
        try {
            const pill = this._proxy.Pill;
            if (pill) {
                this._statusDot.style_class = 'meera-notch-dot';
                if (pill.open_mode && pill.open_mode !== this._openMode) {
                    this._openMode = pill.open_mode;
                    if (this._openMode === 'always') {
                        this._setExpanded(true);
                    }
                }
                if (pill.is_playing === 'true' && pill.surah_title) {
                    this._prayerLabel.text = `${pill.quran_elapsed || '00:00'} · ${pill.surah_title}`;
                    this._countdownLabel.text = 'Tilawah Aktif';
                } else {
                    if (pill.prayer_name) {
                        this._prayerLabel.text = `${pill.prayer_name} ${pill.prayer_time || ''}`.trim();
                    }
                    if (pill.countdown) {
                        this._countdownLabel.text = pill.countdown;
                    }
                }
                if (pill.pomodoro) {
                    this._pomodoroLabel.text = pill.pomodoro;
                }
                if (pill.battery) {
                    this._batteryLabel.text = pill.battery;
                }
            }
        } catch (_) {
            this._setOfflineState();
        }
    }

    destroy() {
        if (this._pollId) {
            GLib.source_remove(this._pollId);
            this._pollId = 0;
        }
        if (this._hoverTimeout) {
            GLib.source_remove(this._hoverTimeout);
            this._hoverTimeout = 0;
        }
        if (this._collapseTimeout) {
            GLib.source_remove(this._collapseTimeout);
            this._collapseTimeout = 0;
        }
        if (this._armedTimer) {
            GLib.source_remove(this._armedTimer);
            this._armedTimer = 0;
        }
        super.destroy();
    }
});
