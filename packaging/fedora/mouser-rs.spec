Name:           mouser-rs
Version:        0.1.0
Release:        1%{?dist}
Summary:        Native Linux daemon and GUI for Logitech HID++ mice and keyboards

License:        MIT
URL:            https://github.com/virajt71/mouse
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  pkgconfig(hidapi-hidraw)
BuildRequires:  pkgconfig(systemd)
BuildRequires:  pkgconfig(gtk+-3.0)

Requires:       hidapi
Requires:       systemd
Requires:       gtk3

%description
A native Linux daemon and GUI for Logitech HID++ mice: button remapping,
gesture control, DPI tuning, SmartShift, and Logitech Flow cross-machine control.

%prep
%autosetup

%build
cargo build --release

%install
rm -rf %{buildroot}
install -d %{buildroot}%{_bindir}
install -d %{buildroot}%{_udevrulesdir}
install -d %{buildroot}%{_datadir}/applications
install -d %{buildroot}%{_userunitdir}

install -m 0755 target/release/mouser-rs %{buildroot}%{_bindir}/mouser-rs
install -m 0644 packaging/common/69-mouser-logitech.rules %{buildroot}%{_udevrulesdir}/69-mouser-logitech.rules
install -m 0644 packaging/common/io.github.mouser.desktop %{buildroot}%{_datadir}/applications/io.github.mouser.desktop
install -d %{buildroot}%{_datadir}/icons/hicolor/scalable/apps
install -m 0644 assets/icons/mouser.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/mouser.svg
install -m 0644 packaging/common/mouser.service %{buildroot}%{_userunitdir}/mouser.service

%files
%{_bindir}/mouser-rs
%{_udevrulesdir}/69-mouser-logitech.rules
%{_datadir}/applications/io.github.mouser.desktop
%{_datadir}/icons/hicolor/scalable/apps/mouser.svg
%{_userunitdir}/mouser.service

%post
%systemd_user_post mouser.service
# Enable for every user account (starts at login)
for d in /home/*/ /root/; do
    u=$(basename "$d")
    getent passwd "$u" >/dev/null 2>&1 && \
        runuser -u "$u" -- systemctl --user enable mouser.service 2>/dev/null || true
done

%preun
%systemd_user_preun mouser.service

%postun
if [ "$1" -eq 0 ]; then
    # Last removal: stop + disable for all users
    for d in /home/*/ /root/; do
        u=$(basename "$d")
        getent passwd "$u" >/dev/null 2>&1 && \
            runuser -u "$u" -- systemctl --user disable --now mouser.service 2>/dev/null || true
    done
fi

%changelog
* Thu Aug 27 2026 Mouser Team <dev@mouser.local> - 0.1.0-1
- Initial release of mouser-rs RPM package.
