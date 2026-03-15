fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/logo.ico");
        res.set("FileDescription", "VaultCanvas");
        res.set("ProductName", "VaultCanvas");
        res.set("CompanyName", "VaultCanvas");
        res.set("InternalName", "VaultCanvas");
        res.set("OriginalFilename", "VaultCanvas.exe");
        res.set("ProductVersion", "1.0.0");
        res.set("FileVersion", "1.0.0");
        res.set(
            "Comments",
            "Local security utility for encryption, decryption and steganography.",
        );
        res.set("LegalCopyright", "Copyright (c) 2026");
        let _ = res.compile();
    }
}
