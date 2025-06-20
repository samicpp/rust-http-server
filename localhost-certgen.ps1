# Create a self-signed cert for localhost, valid for 1 year
$cert = New-SelfSignedCertificate -DnsName "localhost" -CertStoreLocation "cert:\LocalMachine\My" -NotAfter (Get-Date).AddYears(1)

# Export the cert and private key to a PFX file (optional)
$pwd = ConvertTo-SecureString -String "yourpassword" -Force -AsPlainText
Export-PfxCertificate -Cert "cert:\LocalMachine\My\$($cert.Thumbprint)" -FilePath "C:\localhost.pfx" -Password $pwd

# Export public cert to CER file (optional)
Export-Certificate -Cert "cert:\LocalMachine\My\$($cert.Thumbprint)" -FilePath "C:\localhost.cer"
