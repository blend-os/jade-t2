use crate::internal::exec::*;
use crate::internal::files::append_file;
use crate::internal::*;
use log::warn;
use std::path::PathBuf;

pub fn install_base_packages(kernel: String) {
    std::fs::create_dir_all("/mnt/etc").unwrap();
    let kernel_to_install = if kernel.is_empty() {
        "linux"
    } else {
        match kernel.as_str() {
            "linux" => "linux",
            "linux-lts" => "linux-lts",
            "linux-zen" => "linux-zen",
            "linux-hardened" => "linux-hardened",
            _ => {
                warn!("Unknown kernel: {}, using default instead", kernel);
                "linux"
            }
        }
    };
    install::install(vec![
        // Base Arch
        "filesystem-blend",
        "gcc-libs",
        "glibc",
        "bash",
        "zsh",
        "skel-blend",
        "zsh-autosuggestions",
        "zsh-syntax-highlighting",
        "coreutils",
        "file",
        "findutils",
        "gawk",
        "grep",
        "procps-ng",
        "sed",
        "tar",
        "gettext",
        "pciutils",
        "psmisc",
        "shadow",
        "util-linux",
        "bzip2",
        "gzip",
        "xz",
        "licenses",
        "pacman",
        "archlinux-keyring",
        "systemd",
        "systemd-sysvcompat",
        "iputils",
        "iproute2",
        kernel_to_install,
        "linux-firmware",
        "systemd-sysvcompat",
        "networkmanager",
        "man-db",
        "man-pages",
        "texinfo",
        "nano",
        "sudo",
        "curl",
        "archlinux-keyring",
        "blend-keyring",
        "lsb-release-blend",
        // Base blendOS
        "plymouth",
        "blend",
        // Extra goodies
        "neofetch",
        "btrfs-progs",
        "which",
        "base-devel",
        // Fonts
        "noto-fonts",
        "noto-fonts-emoji",
        "noto-fonts-cjk",
        "noto-fonts-extra",
        "ttf-nerd-fonts-symbols-common",
        // Common packages for all desktops
        "xterm",
        "pipewire",
        "pipewire-pulse",
        "pipewire-alsa",
        "pipewire-jack",
        "alsa-firmware",
        "sof-firmware",
        "alsa-ucm-conf",
        "wireplumber",
        "power-profiles-daemon",
        "cups",
        "cups-pdf",
        "bluez",
        "bluez-cups",
        "ntfs-3g",
        "bash-completion",
        "zsh-completions",
        "ttf-liberation",
    ]);
    files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf");
}

pub fn genfstab() {
    exec_eval(
        exec(
            "bash",
            vec![
                String::from("-c"),
                String::from("genfstab -U /mnt >> /mnt/etc/fstab"),
            ],
        ),
        "Generate fstab",
    );
}

pub fn install_bootloader_efi(efidir: PathBuf) {
    install::install(vec![
        "grub",
        "efibootmgr",
        "os-prober",
    ]);
    let efidir = std::path::Path::new("/mnt").join(efidir);
    let efi_str = efidir.to_str().unwrap();
    if !std::path::Path::new(&format!("/mnt{efi_str}")).exists() {
        crash(format!("The efidir {efidir:?} doesn't exist"), 1);
    }
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![
                String::from("--target=x86_64-efi"),
                format!("--efi-directory={}", efi_str),
                String::from("--bootloader-id=blend"),
                String::from("--removable"),
            ],
        ),
        "install grub as efi with --removable",
    );
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![
                String::from("--target=x86_64-efi"),
                format!("--efi-directory={}", efi_str),
                String::from("--bootloader-id=blend"),
            ],
        ),
        "install grub as efi without --removable",
    );
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

pub fn install_bootloader_legacy(device: PathBuf) {
    install::install(vec![
        "grub",
        "os-prober",
    ]);
    if !device.exists() {
        crash(format!("The device {device:?} does not exist"), 1);
    }
    let device = device.to_string_lossy().to_string();
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![String::from("--target=i386-pc"), device],
        ),
        "install grub as legacy",
    );
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

pub fn setup_timeshift() {
    install(vec!["timeshift", "timeshift-autosnap", "grub-btrfs"]);
    exec_eval(
        exec_chroot("timeshift", vec![String::from("--btrfs")]),
        "setup timeshift",
    )
}

pub fn install_homemgr() {
    install(vec!["nix"]);
}

pub fn install_flatpak() {
    install(vec!["flatpak", "flathub-ui"]);
    exec_eval(
        exec_chroot(
            "flatpak",
            vec![
                String::from("remote-add"),
                String::from("--if-not-exists"),
                String::from("flathub"),
                String::from("https://flathub.org/repo/flathub.flatpakrepo"),
            ],
        ),
        "add flathub remote",
    )
}

pub fn install_zram() {
    install(vec!["zram-generator"]);
    files::create_file("/mnt/etc/systemd/zram-generator.conf");
    files_eval(
        files::append_file("/mnt/etc/systemd/zram-generator.conf", "[zram0]"),
        "Write zram-generator config",
    );
}
