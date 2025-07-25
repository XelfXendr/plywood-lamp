set windows-powershell := true

[windows]
run *args:
	cd {{invocation_directory()}}; $Env:SSID = (netsh wlan show interfaces | select-string 'SSID\s+:\s(.+)').Matches.Groups[1].Value.Trim(); $Env:PASSWORD = (netsh wlan show profile name=$Env:SSID key=clear | select-string 'Key Content\s+:\s(.+)').Matches.Groups[1].Value.Trim(); cargo run {{args}}

[windows]
build *args:
	cd {{invocation_directory()}}; $Env:SSID = (netsh wlan show interfaces | select-string 'SSID\s+:\s(.+)').Matches.Groups[1].Value.Trim(); $Env:PASSWORD = (netsh wlan show profile name=$Env:SSID key=clear | select-string 'Key Content\s+:\s(.+)').Matches.Groups[1].Value.Trim(); cargo build {{args}}