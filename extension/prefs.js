import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk';
import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

const BUS_NAME = 'io.github.rizqinrr.MeeraNotch';
const OBJECT_PATH = '/io/github/rizqinrr/MeeraNotch';
const IFACE_NAME = 'io.github.rizqinrr.MeeraNotch1';

function callDaemon(method, variantParams, onDone) {
    try {
        Gio.DBus.session.call(
            BUS_NAME,
            OBJECT_PATH,
            IFACE_NAME,
            method,
            variantParams,
            null,
            Gio.DBusCallFlags.NONE,
            5000,
            null,
            (conn, res) => {
                try {
                    const reply = conn.call_finish(res);
                    if (onDone) onDone(reply);
                } catch (_) {}
            }
        );
    } catch (_) {}
}

export default class MeeraNotchPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        window.set_default_size(520, 440);
        window.set_title('Pengaturan meera-notch');

        const page = new Adw.PreferencesPage({
            title: 'Pengaturan',
            icon_name: 'preferences-system-symbolic',
        });

        const behaviorGroup = new Adw.PreferencesGroup({
            title: 'Perilaku & Tampilan Notch',
            description: 'Atur interaksi pill di bibir atas layar (y = 0)',
        });

        const modeIds = ['hover', 'click', 'always'];
        const modeModel = Gtk.StringList.new(['Hover (Sentuh)', 'Klik', 'Selalu Terbuka']);
        const modeRow = new Adw.ComboRow({
            title: 'Cara membuka hub',
            subtitle: 'Interaksi di bibir atas layar',
            model: modeModel,
            selected: 0,
        });
        modeRow.connect('notify::selected', () => {
            const mode = modeIds[modeRow.selected] || 'hover';
            callDaemon('SetOpenMode', new GLib.Variant('(s)', [mode]));
        });
        behaviorGroup.add(modeRow);

        const fullscreenRow = new Adw.SwitchRow({
            title: 'Sembunyikan saat layar penuh',
            subtitle: 'Otomatis menyembunyikan notch saat video atau aplikasi fullscreen aktif',
            active: true,
        });
        behaviorGroup.add(fullscreenRow);

        const prayerGroup = new Adw.PreferencesGroup({
            title: 'Jadwal Sholat, Tilawah & Lokasi',
            description: 'Metode Kemenag RI (Subuh -20°, Isya -18°, Ihtiyath +2 menit)',
        });

        const locRow = new Adw.ActionRow({
            title: 'Lokasi perhitungan sholat & cuaca',
            subtitle: 'Bandung (-6.9203, 107.6233 · Asia/Jakarta)',
        });
        const detectBtn = new Gtk.Button({
            label: 'Deteksi Ulang IP',
            valign: Gtk.Align.CENTER,
        });
        detectBtn.connect('clicked', () => {
            detectBtn.label = 'Mendeteksi...';
            callDaemon('RefreshLocation', null, (reply) => {
                if (reply) {
                    const [summary] = reply.deep_unpack();
                    locRow.subtitle = summary;
                }
                detectBtn.label = 'Deteksi Ulang IP';
            });
        });
        locRow.add_suffix(detectBtn);
        prayerGroup.add(locRow);

        const qariIds = ['alafasy', 'sudais', 'ghamdi', 'husary'];
        const qariModel = Gtk.StringList.new([
            'Mishary Rashid Alafasy',
            'Abdurrahman As-Sudais',
            'Saad Al-Ghamdi',
            'Mahmoud Khalil Al-Husary',
        ]);
        const qariRow = new Adw.ComboRow({
            title: 'Qari utama',
            subtitle: 'Batas cache lokal: 1 GiB (~/.cache/meera-notch/quran)',
            model: qariModel,
            selected: 0,
        });
        qariRow.connect('notify::selected', () => {
            const qari = qariIds[qariRow.selected] || 'alafasy';
            callDaemon('QuranPlay', new GLib.Variant('(sq)', [qari, 67]));
        });
        prayerGroup.add(qariRow);

        page.add(behaviorGroup);
        page.add(prayerGroup);
        window.add(page);
    }
}
