#!/usr/bin/env pwsh
# Deploy script for Kozak Business game on Solana

Write-Host "🚀 Kozak Business Game - Deployment Script" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Check requirements
Write-Host "`n📋 Checking requirements..." -ForegroundColor Yellow
$requirements = @{
    "solana" = "Solana CLI"
    "anchor" = "Anchor Framework"
    "rustc" = "Rust Compiler"
    "cargo" = "Cargo Package Manager"
}

foreach ($tool in $requirements.GetEnumerator()) {
    $cmd = if ((Get-Command $tool.Key -ErrorAction SilentlyContinue)) { "✅" } else { "❌" }
    Write-Host "$cmd $($tool.Value)"
}

# Configure network
Write-Host "`n🔗 Configuring network..." -ForegroundColor Yellow
$cluster = Read-Host "Choose cluster (devnet/localhost) [devnet]"
if ($cluster -eq "") { $cluster = "devnet" }

solana config set --url $cluster
Write-Host "✅ Network set to: $cluster"

# Get wallet
Write-Host "`n💰 Setting up гаманець..." -ForegroundColor Yellow
$walletPath = "$env:USERPROFILE\.solana\id.json"
if (-not (Test-Path $walletPath)) {
    Write-Host "❌ Гаманець не знайдено на: $walletPath"
    Write-Host "📝 Створюєм новий гаманець..."
    solana-keygen new --outfile $walletPath
}

# Get balance
Write-Host "`n💵 Checking balance..." -ForegroundColor Yellow
solana balance $walletPath

# Request SOL if needed
if ($cluster -eq "devnet") {
    Write-Host "`n⛲ Requesting devnet SOL..." -ForegroundColor Yellow
    solana airdrop 5 $walletPath --url devnet
}

# Build
Write-Host "`n🔨 Building programs..." -ForegroundColor Yellow
anchor build

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Build successful!"

# Deploy
Write-Host "`n📤 Deploying to $cluster..." -ForegroundColor Yellow
anchor deploy --provider.cluster $cluster

Write-Host "`n✅ Deployment complete!" -ForegroundColor Green
Write-Host "`n📋 Next steps:" -ForegroundColor Cyan
Write-Host "1. Update Anchor.toml with Program IDs above"
Write-Host "2. Update README.md with deployment addresses"
Write-Host "3. Run tests: anchor test"
Write-Host "4. Create GitHub Pull Request"
