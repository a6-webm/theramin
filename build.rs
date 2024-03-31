fn main() {
    cc::Build::new()
        .file("manymouse/linux_evdev.c")
        .file("manymouse/macosx_hidmanager.c")
        .file("manymouse/macosx_hidutilities.c")
        .file("manymouse/manymouse.c")
        .file("manymouse/windows_wminput.c")
        .file("manymouse/x11_xinput2.c")
        .define("SUPPORT_XINPUT2", Some("0"))
        .compile("manymouse");
}
