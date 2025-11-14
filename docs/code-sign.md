# Code Signing

When developing software for Windows, it is important to code sign the application to make sure Windows Defender does not treat it as a virus or Unauthorized Application. This Code Signing Certificate acts as an identifier that Windows will use to build a positive reputation for your app.

## Sign Tool GitHub Action

This repo uses [`skymatic/code-sign-action@v1`](https://github.com/marketplace/actions/windows-signtool-exe-code-sign-action) to sign the latest builds of `beans-rs`. The most important components of this action are `certificate`, `certificatesha1` and `password`.

## Generating a Self Signed Certificate on Windows

There is [extensive Microsoft documentation about code signing certificates](https://learn.microsoft.com/en-us/entra/identity-platform/howto-create-self-signed-certificate), however `beans-rs` relies on a script inspired by [this guide](https://archi-lab.net/creating-a-self-signed-code-signing-certificate/).

Before starting the generating certificate process, ensure you have 3 passwords:
- Certificate Authority Password
- Code Signing Certificate Password
- PFX Private Key Password

Place this code in a file called `gen_cert.ps1` and run it in [Developer Powershell for VS 2022](https://learn.microsoft.com/en-us/visualstudio/ide/reference/command-prompt-powershell?view=vs-2022#start-from-windows-menu).

```ps1
# Set Certificate Information
# Replace each entry, including removing brackets.
$FullTeamName = "[replace with full team name]" 
$ShortHandTeamName = "[short hand team name with spaces removed]" 
$FullAppName = "[full app name]"
$AppName = "[app name with spaces removed]"
# NOTE: This process requires 3 separate passwords, Certificate Authority, Code Signing Certificate and PFX Private Key.
# This password should be different than your Certificate Authority and Code Signing Certificate.
$PFXPassword = "[pfx password]"

# Create Certificate Authority
echo "This step uses the first password for the Certificate Authority."
makecert -r -pe -n "CN=$FullTeamName Certificate Authority" -ss CA -sr CurrentUser -a sha256 -cy authority -sky signature -sv $ShortHandTeamName-ca.pvk $ShortHandTeamName-ca.cer
echo "Pleas press Yes to install the certificate."
certutil -user -addstore Root $ShortHandTeamName-ca.cer

# Create Code Signing Certificate
echo "The first two popups are for your second password for the Code Signing Certificate."
echo "The third popup is for your Certificate Authority password from the previous step."
makecert -pe -n "CN=$FullAppName by $FullTeamName" -a sha256 -cy end -sky signature -ic $ShortHandTeamName-ca.cer -iv $ShortHandTeamName-ca.pvk -sv $ShortHandTeamName-$AppName.pvk  $ShortHandTeamName-$AppName.cer
echo "Enter your Code Signing Certificate password from the previous step."
pvk2pfx -pvk $ShortHandTeamName-$AppName.pvk -spc $ShortHandTeamName-$AppName.cer -pfx $ShortHandTeamName-$AppName.pfx -po "$PFXPassword"
echo "Certificate Successfully Created"

# Output Certificate Information
echo "Generating Base64 Hash"
$fileContentBytes = get-content $ShortHandTeamName-$AppName.pfx -Encoding Byte
[System.Convert]::ToBase64String($fileContentBytes) > CODESIGN.txt
echo "Locating Thumbprint"
$StrPassword = ConvertTo-SecureString -String $PFXPassword -Force -AsPlainText
New-Object System.Security.Cryptography.X509Certificates.X509Certificate2("$ShortHandTeamName-$AppName.pfx", "$PFXPassword") > CODESIGN_HASH.txt
echo "Take the data from the CODESIGN.txt add it the repo GitHub Secrets as a secret called CODESIGN."
echo "The long string of numbers under the word Thumbprint is the SHA1 used to identify the certificate. Copy only the string into a secret called CODESIGN_HASH."
echo "Finally take the password entered into PFXPassword and add it as a secret called CODESIGN_PASSWORD."
```
***Update the certificate information before running.***
    
## GitHub Secrets

When creating a fork, visit the `secrets/actions` settings page and add the following entries:

- `CODESIGN`: This is your `certificate` entry in the form of a Base 64 encoded hash from the `.pfx` certificate file. You can recover this from the `CODESIGN.txt` file created when generating the certificate.
- `CODESIGN_PASSWORD`: This is the strong `password` used for protecting the PFX File on Windows. This should be the same password used in `$PFXPassword`.
- `CODESIGN_HASH`: This is the SHA1 "Thumbprint" hash used to identify your `certificatesha1`. You can recover this from the `CODESIGN_HASH.txt` file. You can also find it by [opening Microsoft Management Console and using MMC snap-in to view the certificate](https://learn.microsoft.com/en-us/dotnet/framework/wcf/feature-details/how-to-retrieve-the-thumbprint-of-a-certificate).

## Microsoft Security Intelligence Malware Analysis Submission

After the program has been fully signed submit the application for a [Malware Analysis Submission](https://www.microsoft.com/en-us/wdsi/filesubmission) to certify the authenticity of the software.