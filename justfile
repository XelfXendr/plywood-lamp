set windows-powershell := true

[windows]
run *args:
	#!powershell.exe
	cd {{justfile_directory()}}/lamp-esp32
	$Env:SSID = (netsh wlan show interfaces | select-string 'SSID\s+:\s(.+)').Matches.Groups[1].Value.Trim()
	$Env:PASSWORD = (netsh wlan show profile name=$Env:SSID key=clear | select-string 'Key Content\s+:\s(.+)').Matches.Groups[1].Value.Trim()
	cargo run {{args}}

[windows]
build *args:
	#!powershell.exe
	cd {{justfile_directory()}}/lamp-esp32
	$Env:SSID = (netsh wlan show interfaces | select-string 'SSID\s+:\s(.+)').Matches.Groups[1].Value.Trim()
	$Env:PASSWORD = (netsh wlan show profile name=$Env:SSID key=clear | select-string 'Key Content\s+:\s(.+)').Matches.Groups[1].Value.Trim()
	cargo build {{args}}