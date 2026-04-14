# This file describes Windows specific topics, issues and their workarounds.

## MSVC Toolchain Setup

There are two specifics you have to follow in order to use the MSVC toolchain on Windows:

### 1. Install MSVC Toolchain from Visual Studio

To get the MSVC toolchain you have to install Visual Studio or Visual Studio Build Tools.
Be caurefull as Visual Studio is a different programm than VS Code.

According to the [rustup book](https://rust-lang.github.io/rustup/installation/windows-msvc.html)
you can simply select “Desktop Development with C++” during the installation
which should provide everything needed.

### 2. Adjust PATH environment variable

Most rust packages are able to find the MSVC toolchain automatically,
but for some binaries from the toolchain that is apparently not possible.
E.g. the `cc` crate expects `cl.exe` to be available in `PATH` in case it is unable to find it.
For me, this binary is in the following path:
`C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\ARM64\bin`

Depending on which architecture you are compiling for, you have to add the appropriate path
to the `PATH` environment variable.

## Sign Rust Binaries for Smart App Control

Windows Smart App Control is blocking `cargo.exe`, and potentially other rust utilities
installed via `rustup`,
because I assume they were locally compiled and therefore not signed.
Since disabling Smart App Control decreases your system security,
the recommended workaround is to sign the binaries locally with a trusted self-signed certificate.

### PowerShell script overview

> [!WARNING]
> We will generate a Self-Signed **Code Signing Certificate**.
> To make Windows trust this certificate (and thus allow running the signed binaries), we must add it to the **Trusted Root Certification Authorities** and **Trusted Publishers** certificate stores for your user account. Since adding certificates to the Trusted Root can have security implications, it requires your explicit approval.
> The certificate will only be used by you, locally, to sign these specific binaries.

#### 1. Certificate Creation and Trusting
We will run the following PowerShell commands:
```powershell
# Create code signing cert
$cert = New-SelfSignedCertificate -Subject "CN=RustLocalDevCodeSign" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"

# Trust the cert in Root
$storeRoot = Get-Item "Cert:\CurrentUser\Root"
$storeRoot.Open("ReadWrite")
$storeRoot.Add($cert)
$storeRoot.Close()

# Trust the cert in TrustedPublisher
$storePub = Get-Item "Cert:\CurrentUser\TrustedPublisher"
$storePub.Open("ReadWrite")
$storePub.Add($cert)
$storePub.Close()
```

#### 2. Sign the Binaries
We will sign all executables in your CARGO_HOME directory (including `cargo.exe`, `rustc.exe`, `rustup.exe`, etc.):
```powershell
$binaries = Get-ChildItem -Path "$env:USERPROFILE\.cargo\bin\*.exe"
foreach ($bin in $binaries) {
    Set-AuthenticodeSignature -FilePath $bin.FullName -Certificate $cert
}
```

Furthermore, we also have to sign all binaries in the rustup toolchains directory as the binaries in the cargo folder are just shortcuts. (See the complete script for the exact commands)

#### 3. Usage

It may be necessary to execute the script as Administrator for the certificate creation but signing can be done as a regular user.
Make also sure that the execution policy is allowing you to execute the script.
You can check it with `Get-ExecutionPolicy` and set it with `Set-ExecutionPolicy`.

**To sign all standard Rust binaries:**
```powershell
.\sign_rust_binaries.ps1
```

**To sign a newly compiled binary that gets blocked:**
```powershell
.\sign_rust_binaries.ps1 .\target\debug\my_app.exe
```
