fn main() {
    // Embed Windows application manifest for DPI awareness and visual styles
    let manifest = r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="1.0.0.0"
    processorArchitecture="*"
    name="RustTaskManager"
    type="win32"
  />
  <description>Native High-Performance Task Manager</description>
  
  <!-- Per-Monitor DPI v2 Awareness -->
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
  
  <!-- Visual Styles (Fluent Design) -->
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
  
  <!-- Execution Level (start as standard user, elevate on-demand) -->
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  
  <!-- Windows 10+ compatibility -->
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 1809+ -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
      <!-- Windows 11 -->
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
    </application>
  </compatibility>
</assembly>
"#;

    // Write manifest to OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let manifest_path = std::path::Path::new(&out_dir).join("app.manifest");
    std::fs::write(&manifest_path, manifest).expect("Failed to write manifest");

    // Embed manifest in executable
    println!("cargo:rerun-if-changed=build.rs");

    // For MSVC, embed the manifest using embed-resource
    if cfg!(target_os = "windows") {
        // Write a simple resource file that embeds the manifest
        let rc_path = std::path::Path::new(&out_dir).join("app.rc");

        // Use relative path for the RC file
        let rc_content = "1 24 \"app.manifest\"";
        std::fs::write(&rc_path, rc_content).expect("Failed to write .rc file");

        embed_resource::compile(&rc_path, embed_resource::NONE);
    }
}
