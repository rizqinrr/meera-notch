import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {NotchWidget} from './notch.js';

export default class MeeraNotchExtension extends Extension {
    enable() {
        this._notch = new NotchWidget();

        Main.layoutManager.addTopChrome(this._notch, {
            trackFullscreen: true,
            affectsStruts: false,
            affectsInputRegion: true,
        });

        this._reposition = () => this._positionNotch();
        this._monitorsChangedId = Main.layoutManager.connect('monitors-changed', this._reposition);
        this._notchAllocId = this._notch.connect('notify::allocation', this._reposition);
        this._positionNotch();
    }

    _positionNotch() {
        if (!this._notch) return;
        const monitor = Main.layoutManager.primaryMonitor;
        if (!monitor) return;
        const [, natW] = this._notch.get_preferred_width(-1);
        const w = natW || this._notch.width || 308;
        const x = monitor.x + Math.floor((monitor.width - w) / 2);
        const y = monitor.y;
        this._notch.set_position(x, y);
    }

    disable() {
        if (this._monitorsChangedId) {
            Main.layoutManager.disconnect(this._monitorsChangedId);
            this._monitorsChangedId = 0;
        }
        if (this._notch && this._notchAllocId) {
            this._notch.disconnect(this._notchAllocId);
            this._notchAllocId = 0;
        }
        if (this._notch) {
            Main.layoutManager.removeChrome(this._notch);
            this._notch.destroy();
            this._notch = null;
        }
    }
}
