[CmdletBinding()]
param(
 [string]$PorchDirectory = "$env:LOCALAPPDATA\InfinitePorch\0.1.1",
 [string]$Data = "$env:LOCALAPPDATA\InfinitePorch\state",
 [Parameter(Mandatory=$true)][string]$Peer,
 [Parameter(Mandatory=$true)][string]$Model,
 [ValidateSet('baseline','revoked','restart','offline','restored')][string]$Phase = 'baseline',
 [Parameter(Mandatory=$true)][string]$EvidenceDirectory,
 [switch]$SeparateMachinesConfirmed, [switch]$WanConditionConfirmed, [switch]$Message
)
$ErrorActionPreference = 'Stop'
if (!$SeparateMachinesConfirmed) { throw 'Confirm independently owned, separate physical machines with -SeparateMachinesConfirmed.' }
$PorchCli = "$PorchDirectory\bin\porch.exe"
$PorchArguments = @('--data',$Data,'qualify','run','--peer',$Peer,'--model',$Model,'--environment','PHYSICAL','--phase',$Phase,'--separate-machines-confirmed')
if ($WanConditionConfirmed) { $PorchArguments += '--wan-condition-confirmed' }
if ($Message) { $PorchArguments += '--message' }
& $PorchCli @PorchArguments
if ($LASTEXITCODE -ne 0) { throw 'Qualification command failed' }
& $PorchCli --data $Data qualify export $EvidenceDirectory
if ($LASTEXITCODE -ne 0) { throw 'Evidence export failed' }
& $PorchCli qualify validate $EvidenceDirectory
if ($LASTEXITCODE -ne 0) { throw 'Evidence validation failed' }
