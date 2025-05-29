Name:           smoldesk
Version:        1.0.0
Release:        1%{?dist}
Summary:        WebRTC-based Remote Desktop for Linux

License:        MIT
URL:            https://github.com/SmolDesk/SmolDesk
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  nodejs >= 16
BuildRequires:  npm
BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  webkit2gtk4.1-devel
BuildRequires:  gtk3-devel
BuildRequires:  pkg-config

Requires:       webkit2gtk4.1
Requires:       gtk3
Requires:       ffmpeg
Requires:       wl-clipboard
Requires:       ydotool

%description
SmolDesk is a modern remote desktop solution that provides
low-latency screen sharing using WebRTC technology.
Supports both X11 and Wayland display servers.

%prep
%setup -q

%build
npm install
npm run tauri build

%install
mkdir -p %{buildroot}/opt/smoldesk
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/pixmaps

# Install binary
cp src-tauri/target/release/smoldesk %{buildroot}/opt/smoldesk/
ln -s /opt/smoldesk/smoldesk %{buildroot}/usr/bin/smoldesk

# Install desktop file and icon
cp packaging/smoldesk.desktop %{buildroot}/usr/share/applications/
cp docs/static/img/logo.png %{buildroot}/usr/share/pixmaps/smoldesk.png

# Install signaling server
mkdir -p %{buildroot}/opt/smoldesk/signaling-server
cp -r signaling-server/* %{buildroot}/opt/smoldesk/signaling-server/

%post
# Update desktop database
/usr/bin/update-desktop-database &> /dev/null || :

# Create udev rules
cat > /etc/udev/rules.d/99-smoldesk.rules << 'EOF'
KERNEL=="uinput", GROUP="input", MODE="0660"
SUBSYSTEM=="input", GROUP="input", MODE="0664"
EOF

# Reload udev
/usr/bin/udevadm control --reload-rules &> /dev/null || :
/usr/bin/udevadm trigger &> /dev/null || :

%postun
if [ $1 -eq 0 ] ; then
    /usr/bin/update-desktop-database &> /dev/null || :
fi

%files
/opt/smoldesk/smoldesk
/usr/bin/smoldesk
/usr/share/applications/smoldesk.desktop
/usr/share/pixmaps/smoldesk.png
/opt/smoldesk/signaling-server/

%changelog
* Wed May 29 2025 SmolDesk Team <team@smoldesk.example> - 1.0.0-1
- Initial release
