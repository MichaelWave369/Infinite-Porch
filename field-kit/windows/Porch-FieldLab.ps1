[CmdletBinding()]
param(
    [Parameter(Position=0,Mandatory)][ValidateSet('setup','pair-host','pair-join','model-host','model-join','storage-host','qualify','offline','restart-check','revoke','refused-check','snapshot','correlate','failure-check','stop')][string]$Command,
    [string]$Alias, [string]$State = "$env:LOCALAPPDATA\InfinitePorch\field-state",
    [string]$PackageRoot = (Resolve-Path "$PSScriptRoot\..\..").Path,
    [string]$Peer, [string]$Model, [string]$File, [string]$EvidenceB, [string]$Out,
    [string]$Fingerprint, [string]$Session = 'porch-field', [string]$PorchName='Infinite Porch Field Lab',
    [ValidateRange(1024,65535)][int]$ApiPort=7331,
    [ValidateRange(1024,65535)][int]$PeerPort=7332,
    [ValidateSet('PHYSICAL_LAN','NATIVE_HOSTED','LOOPBACK')][string]$Environment='PHYSICAL_LAN',
    [switch]$Guided, [switch]$SeparateMachinesConfirmed, [switch]$WanConditionConfirmed, [switch]$NoMdns
)
. "$PSScriptRoot\Field-Common.ps1"
$manifest = Test-PorchPackage $PackageRoot
$cli = Join-Path $PackageRoot 'bin\porch.exe'
$daemon = Join-Path $PackageRoot 'bin\porch-node.exe'
$api = "http://127.0.0.1:$ApiPort"
$settings = Join-Path $State 'field-settings.json'
if ($env:GITHUB_ACTIONS -eq 'true' -and $Environment -eq 'PHYSICAL_LAN') { throw 'Hosted runner cannot execute a physical classification' }
if ($Session -notmatch '^[A-Za-z0-9_-]{1,48}$') { throw 'Use a short public session label shared by both PCs' }
function Invoke-Porch([string[]]$Arguments) {
    $text = & $cli --data $State --api $api @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Porch command refused: $($Arguments[0]) (exit $LASTEXITCODE)" }
    $text -join "`n" | ConvertFrom-Json
}
function Invoke-Operation([string]$Operation, $Arguments) { Invoke-Porch @('call',$Operation,'--json',($Arguments | ConvertTo-Json -Depth 40 -Compress)) }
function Save-Field($Value,[string]$Path) {
    if (Test-Path -LiteralPath $Path) { throw 'Evidence already exists; select a fresh -Out' }
    $parent = Split-Path -Parent $Path
    if ($parent) { New-Item -ItemType Directory -Force $parent | Out-Null }
    $Value | ConvertTo-Json -Depth 80 | Set-Content -LiteralPath $Path -Encoding utf8
}
function Get-Checkpoint { Invoke-Porch @('share','refresh') | Out-Null; Invoke-Operation 'field.snapshot' @{session=$Session;environment=$Environment;separate_machines_confirmed=[bool]$SeparateMachinesConfirmed} }
function Start-FieldNode {
    try { return Invoke-Porch @('status') } catch {}
    if (Test-Path $settings) {
        $s=Get-Content $settings -Raw | ConvertFrom-Json
        if ($s.api_port -ne $ApiPort -or $s.peer_port -ne $PeerPort) { throw 'Saved field ports differ; reuse setup ports' }
    }
    $listen='/ip4/0.0.0.0'; if ($Environment -ne 'PHYSICAL_LAN') { $listen='/ip4/127.0.0.1' }
    $args=@('--data',('"'+$State+'"'),'--api',"127.0.0.1:$ApiPort",'--listen',"$listen/tcp/$PeerPort",'--listen',"$listen/udp/$PeerPort/quic-v1",'--ui',('"'+(Join-Path $PackageRoot 'ui')+'"'))
    if ($NoMdns) { $args+='--no-mdns' }
    $process=Start-Process -FilePath $daemon -ArgumentList $args -PassThru -RedirectStandardOutput (Join-Path $State 'field-node.stdout.log') -RedirectStandardError (Join-Path $State 'field-node.stderr.log')
    @{pid=$process.Id;path=$daemon;started=$process.StartTime.ToUniversalTime().ToString('o')} | ConvertTo-Json | Set-Content (Join-Path $State 'field-owned-process.json')
    for ($i=0;$i -lt 100;$i++) { Start-Sleep -Milliseconds 150; try { return Invoke-Porch @('status') } catch { if ($process.HasExited) { throw 'Owned Porch node exited; preserve local logs' } } }
    throw 'Node did not become ready'
}
function Stop-FieldNode {
    $file=Join-Path $State 'field-owned-process.json'
    if (!(Test-Path $file)) { throw 'No field-owned process record; stop a manually launched node yourself' }
    $owned=Get-Content $file -Raw | ConvertFrom-Json
    $p=Get-Process -Id $owned.pid -ErrorAction Stop
    if ($p.Path -ne $owned.path -or $p.StartTime.ToUniversalTime().ToString('o') -ne $owned.started) { throw 'PID no longer identifies the field-owned node' }
    Stop-Process -Id $p.Id
    $p.WaitForExit(8000) | Out-Null
    Remove-Item $file
}
function Confirm-Identity([string]$Description) {
    Write-Host $Description
    if ((Read-Host 'Type CONFIRM after comparing identities on both intended machines') -cne 'CONFIRM') { throw 'Operator confirmation required' }
}
function Run-Field([string]$Phase) {
    if (!$Peer -or !$Model) { throw '-Peer and exact -Model required' }
    if ($Environment -eq 'PHYSICAL_LAN' -and !$SeparateMachinesConfirmed) { throw '-SeparateMachinesConfirmed required' }
    $before=Invoke-Porch @('network')
    $args=@('qualify','run','--peer',$Peer,'--model',$Model,'--session',$Session,'--environment',$Environment,'--phase',$Phase,'--message')
    if ($Phase -eq 'baseline') { $args+='--storage' }
    if ($SeparateMachinesConfirmed) { $args+='--separate-machines-confirmed' }
    if ($WanConditionConfirmed) { $args+='--wan-condition-confirmed' }
    $run=Invoke-Porch $args
    if (!$Out) { $script:Out=Join-Path (Get-Location) ("evidence-$Session-$Phase-"+[DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfff')) }
    New-Item -ItemType Directory $Out | Out-Null
    Save-Field $run (Join-Path $Out 'run.json')
    Save-Field (Get-Checkpoint) (Join-Path $Out 'participant.json')
    $run.payload.scenarios | Select-Object id,result | Format-Table | Out-String | Write-Host
    $failed=@($run.payload.scenarios | Where-Object result -eq 'FAIL')
    if ($failed.Count) { throw "Qualification retained $($failed.Count) failed scenario(s) in $Out" }
    Write-Host "Evidence: $Out. After owner revocation run refused-check, then restart-check, then snapshot on both PCs."
    return $before
}
switch ($Command) {
    'setup' {
        if (!$Alias) { throw '-Alias PC-A or PC-B required' }
        if ($ApiPort -eq $PeerPort -or $PeerPort -eq 11434) { throw 'Peer port must differ from both control APIs' }
        New-Item -ItemType Directory -Force $State | Out-Null
        $acl=Protect-PorchState $State
        $before=$null; if (Test-Path (Join-Path $State 'identity.key')) { $before=(Get-FileHash (Join-Path $State 'identity.key')).Hash }
        Invoke-Porch @('init','--alias',$Alias) | Out-Null
        if ($before -and $before -ne (Get-FileHash (Join-Path $State 'identity.key')).Hash) { throw 'Identity changed unexpectedly' }
        Protect-PorchState $State | Out-Null
        @{api_port=$ApiPort;peer_port=$PeerPort;alias=$Alias;environment=$Environment;session=$Session} | ConvertTo-Json | Set-Content $settings
        $status=Start-FieldNode
        $gateFile=Join-Path $PackageRoot 'qualification-gates.json'
        if (Test-Path $gateFile) { Invoke-Porch @('qualify','import-gates',$gateFile) | Out-Null }
        Write-Host "Alias $($status.node.alias) | Peer $($status.node.id) | Version 0.1.2 unsigned"
        $acl | ConvertTo-Json | Write-Output
        Invoke-Porch @('models','scan') | ConvertTo-Json -Depth 12 | Write-Output
        Invoke-Porch @('doctor') | ConvertTo-Json -Depth 12 | Write-Output
        & "$PSScriptRoot\firewall-status.ps1"
        Write-Host 'Next: compare identity show on both PCs; PC-A pair-host -Peer PC-B-ID -File invite.json; PC-B pair-join -File invite.json -Fingerprint HOST-SHA256. Use the same -Session and setup ports on each subsequent command.'
    }
    'pair-host' {
        if (!$Peer -or !$File) { throw '-Peer and -File invitation path required' }
        $id=Invoke-Porch @('identity','show');$id | ConvertTo-Json | Write-Host
        Write-Host ('Local fingerprint phrase: '+(Get-PorchFingerprintPhrase $id.fingerprint_sha256))
        Write-Host 'Phrase is a 96-bit comparison aid; full SHA-256 remains mandatory for joining.'
        Confirm-Identity "Expected recipient: $Peer; local alias: $((Invoke-Porch @('status')).node.alias)"
        Invoke-Porch @('qualify','host',$PorchName,'--recipient',$Peer,'--out',$File) | ConvertTo-Json -Depth 16 | Write-Output
        Write-Host "Invitation: $File. Compare full host SHA-256 fingerprint with PC-B; invitation expires in ten minutes."
    }
    'pair-join' {
        if (!$File -or !$Fingerprint) { throw '-File and -Fingerprint verified on PC-A required' }
        $invite=Get-Content $File -Raw | ConvertFrom-Json
        Write-Host "Host $($invite.signer) | Porch $($invite.payload.porch) | Expected recipient $($invite.payload.recipient)"
        Invoke-Porch @('identity','show') | ConvertTo-Json | Write-Host
        Write-Host ('Host fingerprint phrase: '+(Get-PorchFingerprintPhrase $Fingerprint))
        Write-Host ('Local alias: '+(Invoke-Porch @('status')).node.alias)
        $local=Invoke-Porch @('identity','show');Write-Host ('Local fingerprint phrase: '+(Get-PorchFingerprintPhrase $local.fingerprint_sha256))
        Confirm-Identity "Host fingerprint: $Fingerprint"
        Invoke-Porch @('qualify','join',$File,'--fingerprint',$Fingerprint) | ConvertTo-Json -Depth 16 | Write-Output
    }
    'model-host' {
        if (!$Peer -or !$Out) { throw '-Peer and fresh -Out grant file required' }
        $scan=Invoke-Porch @('models','scan');$scan | ConvertTo-Json -Depth 16 | Write-Host
        if (!$Model) { $script:Model=Read-Host 'Exact installed model name from inventory' }
        $verified=Invoke-Porch @('models','verify',$Model)
        if ($verified.status -ne 'COMPLETED' -or $verified.receipt.payload.provider -ne 'ollama') { throw 'Real installed Ollama invocation required' }
        Invoke-Porch @('share','model',$Model) | Out-Null
        Invoke-Porch @('grant','issue','--recipient',$Peer,'--capability','model.inference','--resource',$Model,'--ttl','3600','--max-calls','12','--calls-per-hour','12','--max-output-tokens','16','--max-duration-ms','30000','--out',$Out) | Out-Null
        $base=Split-Path $Out -Parent; if (!$base) { $base='.' }
        Invoke-Porch @('grant','issue','--recipient',$Peer,'--capability','message.direct','--resource','inbox','--action','send','--max-calls','20','--calls-per-hour','20','--out',(Join-Path $base 'message-grant.json')) | Out-Null
        Write-Host "Installed model verified and exposed. Grant: $Out; message-grant.json in same directory. Run storage-host before requester qualify."
    }
    'storage-host' {
        if (!$Peer -or !$Out) { throw '-Peer and fresh -Out grant file required' }
        Invoke-Porch @('share','storage','--limit','2MiB') | Out-Null
        Invoke-Porch @('grant','issue','--recipient',$Peer,'--capability','blob.storage','--resource','vault','--action','store','--ttl','3600','--max-calls','20','--calls-per-hour','20','--max-storage-bytes','1MiB','--out',$Out) | Out-Null
        Write-Host 'Transfer storage grant to requester and import using model-join -File this-grant.json.'
    }
    'model-join' {
        if (!$File) { throw '-File grant required' }
        Invoke-Porch @('grant','import',$File) | Out-Null
        Invoke-Porch @('resources') | Out-Null
        Invoke-Porch @('share','refresh') | Out-Null
        Invoke-Porch @('models','list') | ConvertTo-Json -Depth 20 | Write-Output
        Invoke-Porch @('grants') | ConvertTo-Json -Depth 20 | Write-Output
        Write-Host 'VISIBLE, REACHABLE, AUTHORIZED, VERIFIED and SUCCESSFULLY_EXECUTED remain separate observations. Import each model, message and storage grant.'
    }
    'qualify' {
        if (!$Guided) { Run-Field 'baseline' | Out-Null; break }
        if (!$Out) { $script:Out=Join-Path (Get-Location) ("guided-"+$Session+'-'+[DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfff')) }
        $guidedRoot=$Out
        if (Test-Path $guidedRoot) { throw 'Guided evidence root already exists; choose fresh -Out' }
        New-Item -ItemType Directory $guidedRoot | Out-Null
        $events=@()
        $script:Out=Join-Path $guidedRoot 'baseline'
        try { Run-Field 'baseline' | Out-Null } catch { $events+=@{phase='baseline';result='FAIL';reason=$_.Exception.Message};Write-Warning $_.Exception.Message }
        Confirm-Identity 'On PC-A run revoke -File model-grant.json for this requester. Confirm only after PC-A reports revocation.'
        $script:Out=Join-Path $guidedRoot 'revoked'
        try { Run-Field 'revoked' | Out-Null } catch { $events+=@{phase='revoked';result='FAIL';reason=$_.Exception.Message};Write-Warning $_.Exception.Message }
        try {
            $before=Get-Checkpoint;Stop-FieldNode;Start-FieldNode | Out-Null
            $restart=Invoke-Operation 'field.restart' @{before=$before}
            Save-Field $restart (Join-Path $guidedRoot 'restart.json')
            if ($restart.payload.scenarios[0].result -ne 'PASS') { throw 'Restart persistence scenario failed' }
        } catch { $events+=@{phase='restart';result='FAIL';reason=$_.Exception.Message};Write-Warning $_.Exception.Message }
        Save-Field (Get-Checkpoint) (Join-Path $guidedRoot 'requester-participant.json')
        Write-Host 'PC-A: run restart-check, then snapshot to PC-A-participant.json using this same session. Transfer that signed snapshot here.'
        $remoteFile=Read-Host 'Path to the independently exported PC-A participant JSON'
        & $cli qualify correlate $remoteFile (Join-Path $guidedRoot 'requester-participant.json') --out (Join-Path $guidedRoot 'report')
        if ($LASTEXITCODE) { $events+=@{phase='correlate';result='FAIL';reason='Participant validation failed'} }
        Save-Field @{events=$events;failed_events=$events.Count;no_absent_step_counts_as_pass=$true} (Join-Path $guidedRoot 'guided-events.json')
        Write-Host "Guided evidence retained: $guidedRoot. Read report state; a command completing does not imply physical qualification."
        if ($events.Count) { throw 'Guided run retained failures; inspect evidence and report' }
    }
    'offline' {
        if (!$WanConditionConfirmed) { throw 'Human must remove only upstream Internet and confirm -WanConditionConfirmed; keep both PCs on the LAN' }
        $pre=Invoke-Porch @('network')
        Write-Host 'Observe the router upstream link. No adapter, service or router configuration will be changed.'
        $external=@(); foreach ($hostName in @('1.1.1.1','8.8.8.8')) {
            $client=[Net.Sockets.TcpClient]::new();$reachable=$false
            try { $task=$client.ConnectAsync($hostName,443);$reachable=$task.Wait(2500) -and $client.Connected } catch {} finally { $client.Dispose() }
            $external+=@{host=$hostName;port=443;reachable=$reachable;note='Failed connection alone cannot prove Internet absence'}
        }
        Run-Field 'offline' | Out-Null
        Save-Field @{environment=$Environment;wan_condition_attested=$true;pre_offline_peer_path=$pre;offline_peer_path=(Invoke-Porch @('network'));external_observations=$external;timestamp=[DateTime]::UtcNow.ToString('o');automatically_proven=$false} (Join-Path $Out 'wan-observation.json')
    }
    'restart-check' {
        $before=Get-Checkpoint
        Stop-FieldNode;Start-FieldNode | Out-Null
        $result=Invoke-Operation 'field.restart' @{before=$before}
        if (!$Out) { throw '-Out fresh evidence directory required' }
        New-Item -ItemType Directory $Out | Out-Null
        Save-Field $result (Join-Path $Out 'restart.json');Save-Field (Get-Checkpoint) (Join-Path $Out 'participant.json')
        $result.payload.scenarios | ConvertTo-Json -Depth 12 | Write-Output
        if ($result.payload.scenarios[0].result -ne 'PASS') { throw 'Restart persistence failed; evidence retained' }
    }
    'revoke' {
        if (!$File) { throw '-File original model grant required' }
        $grant=Get-Content $File -Raw | ConvertFrom-Json
        Invoke-Porch @('grant','revoke',$grant.payload.nonce) | ConvertTo-Json | Write-Output
        Write-Host 'Requester now runs refused-check; export fresh snapshots on both PCs.'
    }
    'refused-check' { Run-Field 'revoked' | Out-Null }
    'snapshot' { if (!$Out) { throw '-Out participant JSON file required' };Save-Field (Get-Checkpoint) $Out;Write-Host "Signed participant snapshot: $Out" }
    'correlate' { if (!$File -or !$EvidenceB -or !$Out) { throw '-File evidence-A.json -EvidenceB evidence-B.json -Out fresh-report required' }; & $cli qualify correlate $File $EvidenceB --out $Out; if ($LASTEXITCODE) { throw 'Evidence validation or correlation failed' } }
    'failure-check' {
        if (!$Peer) { throw '-Peer required' }
        $r=Invoke-Operation 'job.run' @{resource='porch-nonexistent-field-model';input='Infinite Porch public qualification probe. Reply briefly with PORCH_OK.';privacy='TRUSTED_PEERS';preferred_peer=$Peer;max_output_tokens=16}
        if ($r.status -ne 'REFUSED') { throw 'Nonexistent model was not refused' }
        Write-Host 'PASS nonexistent model refusal. Other optional checks are documented in FAILURE_CHECKS.md.'
    }
    'stop' { Stop-FieldNode;Write-Host 'Only the recorded field-owned Porch process was stopped.' }
}
