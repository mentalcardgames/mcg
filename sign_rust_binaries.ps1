param (
    [string]$FilePath = ""
)

$certSubject = "RustLocalDevCodeSign"
$cert = Get-ChildItem -Path "Cert:\CurrentUser\My" | Where-Object { $_.Subject -match $certSubject } | Select-Object -First 1

if (-not $cert) {
    Write-Host "Creating new self-signed code signing certificate..."
    $cert = New-SelfSignedCertificate -Subject "CN=$certSubject" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
    
    Write-Host "Trusting certificate in Root..."
    $storeRoot = Get-Item "Cert:\CurrentUser\Root"
    $storeRoot.Open("ReadWrite")
    $storeRoot.Add($cert)
    $storeRoot.Close()
    
    Write-Host "Trusting certificate in TrustedPublisher..."
    $storePub = Get-Item "Cert:\CurrentUser\TrustedPublisher"
    $storePub.Open("ReadWrite")
    $storePub.Add($cert)
    $storePub.Close()
} else {
    Write-Host "Certificate already exists: $($cert.Thumbprint)"
}

function Sign-BinaryIfNotSigned {
    param (
        [Parameter(Mandatory=$true)]
        [System.IO.FileInfo]$FileInfo
    )
    
    $sig = Get-AuthenticodeSignature -FilePath $FileInfo.FullName
    if ($sig.Status -eq "Valid") {
        Write-Host "Already signed: $($FileInfo.FullName)"
    } else {
        Write-Host "Signing: $($FileInfo.FullName)"
        Set-AuthenticodeSignature -FilePath $FileInfo.FullName -Certificate $cert
    }
}

if ($FilePath) {
    if (Test-Path $FilePath) {
        $fileInfo = Get-Item -Path $FilePath
        Sign-BinaryIfNotSigned -FileInfo $fileInfo
    } else {
        Write-Host "Error: Cannot find file '$FilePath'"
        exit 1
    }
} else {
    Write-Host "No specific file provided. Scanning standard Cargo and Rustup directories..."
    
    $cargoDir = if ($env:CARGO_HOME) { Join-Path $env:CARGO_HOME "bin" } else { "$env:USERPROFILE\.cargo\bin" }
    if (Test-Path $cargoDir) {
        Write-Host "Scanning Cargo directory: $cargoDir"
        $binaries = Get-ChildItem -Path $cargoDir -Filter "*.exe"
        foreach ($bin in $binaries) {
            Sign-BinaryIfNotSigned -FileInfo $bin
        }
    } else {
        Write-Host "Cargo bin directory not found at $cargoDir."
    }
    
    $rustupHome = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { "$env:USERPROFILE\.rustup" }
    $rustupDir = Join-Path $rustupHome "toolchains"
    if (Test-Path $rustupDir) {
        Write-Host "Scanning Rustup toolchains directory: $rustupDir"
        $binaries = Get-ChildItem -Path $rustupDir -Filter "*.exe" -Recurse
        foreach ($bin in $binaries) {
            Sign-BinaryIfNotSigned -FileInfo $bin
        }
    } else {
        Write-Host "Rustup toolchains directory not found at $rustupDir."
    }
}

Write-Host "Done."
