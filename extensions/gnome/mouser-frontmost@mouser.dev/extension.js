// Mouser Frontmost Window — GNOME Shell extension.
//
// Exports a tiny D-Bus service that returns the WM_CLASS and PID of the
// currently focused window. Mouser's `gnome_shell` frontmost backend polls
// this to drive per-app mouse-profile switching on GNOME Wayland.
//
// ESM module style; targets GNOME Shell 45+.

import Gio from 'gi://Gio';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const DBUS_NAME = 'org.mouser.Frontmost';
const DBUS_PATH = '/org/mouser/Frontmost';
const DBUS_INTERFACE = `
<node>
  <interface name="org.mouser.Frontmost">
    <method name="GetFocusedWmClass">
      <arg type="s" direction="out" name="wmClass"/>
    </method>
    <method name="GetFocusedWindowInfo">
      <arg type="s" direction="out" name="wmClass"/>
      <arg type="u" direction="out" name="pid"/>
    </method>
  </interface>
</node>`;

export default class MouserFrontmostExtension extends Extension {
    enable() {
        this._dbus = Gio.DBusExportedObject.wrapJSObject(DBUS_INTERFACE, this);
        this._dbus.export(Gio.DBus.session, DBUS_PATH);
        this._nameId = Gio.bus_own_name_on_connection(
            Gio.DBus.session,
            DBUS_NAME,
            Gio.BusNameOwnerFlags.NONE,
            null,
            null);
    }

    disable() {
        if (this._nameId) {
            Gio.bus_unown_name(this._nameId);
            this._nameId = 0;
        }
        if (this._dbus) {
            this._dbus.unexport();
            this._dbus = null;
        }
    }

    // Legacy method
    GetFocusedWmClass() {
        const win = global.display.focus_window;
        if (!win)
            return '';
        return win.get_wm_class() || '';
    }

    // New unified method returning both app class and process ID
    GetFocusedWindowInfo() {
        const win = global.display.focus_window;
        if (!win)
            return ['', 0];
        return [win.get_wm_class() || '', win.get_pid() || 0];
    }
}
