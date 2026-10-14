# Script to batch generate all 3D print assets using OpenSCAD
param (
    [string]$OpenScad = "C:\Users\arunj\openscad_portable\openscad-2021.01\openscad.com",
    [string]$ScadFile = "$PSScriptRoot\longan_nano_ina219_case.scad",
    [string]$OutDir   = "$PSScriptRoot"
)

Write-Host "=== Generating 3D Enclosure Assets for Longan Nano + INA219 ===" -ForegroundColor Cyan

# 1. Base (STL + 3MF)
Write-Host "[1/6] Exporting Base (STL & 3MF)..." -ForegroundColor Yellow
& $OpenScad -D mode=base --export-format binstl -o "$OutDir\case_base.stl" -o "$OutDir\case_base.3mf" "$ScadFile"

# 2. Lid (STL + 3MF - Default Orientation)
Write-Host "[2/6] Exporting Lid - Default Orientation (STL & 3MF)..." -ForegroundColor Yellow
& $OpenScad -D mode=lid --export-format binstl -o "$OutDir\case_lid.stl" -o "$OutDir\case_lid.3mf" "$ScadFile"

# 3. Lid Print-Ready (STL + 3MF - Flipped 180 deg for support-free bed printing)
Write-Host "[3/6] Exporting Lid - Printable Orientation (STL & 3MF)..." -ForegroundColor Yellow
& $OpenScad -D mode=lid_print --export-format binstl -o "$OutDir\case_lid_print.stl" -o "$OutDir\case_lid_print.3mf" "$ScadFile"

# 4. Exploded CAD Model (STL)
Write-Host "[4/6] Exporting Exploded CAD Model (STL)..." -ForegroundColor Yellow
& $OpenScad -D mode=exploded --export-format binstl -o "$OutDir\case_exploded.stl" "$ScadFile"

# 5. Assembled CAD Model (STL)
Write-Host "[5/6] Exporting Assembled CAD Model (STL)..." -ForegroundColor Yellow
& $OpenScad -D mode=assembled --export-format binstl -o "$OutDir\case_assembled.stl" "$ScadFile"

# 6. Dual Bed Print (Both Parts on Build Plate)
Write-Host "[6/6] Exporting Combined Print Bed (STL & 3MF)..." -ForegroundColor Yellow
& $OpenScad -D mode=print --export-format binstl -o "$OutDir\case_both_print.stl" -o "$OutDir\case_both_print.3mf" "$ScadFile"

# Previews
Write-Host "Rendering High-Res Preview Images..." -ForegroundColor Yellow
& $OpenScad -D mode=exploded --preview --viewall --autocenter --imgsize=1024,768 -o "$OutDir\preview_exploded.png" "$ScadFile"
& $OpenScad -D mode=assembled --preview --viewall --autocenter --imgsize=1024,768 -o "$OutDir\preview_assembled.png" "$ScadFile"
& $OpenScad -D mode=base --preview --viewall --autocenter --imgsize=1024,768 -o "$OutDir\preview_base.png" "$ScadFile"
& $OpenScad -D mode=lid --preview --viewall --autocenter --imgsize=1024,768 -o "$OutDir\preview_lid.png" "$ScadFile"
& $OpenScad -D mode=lid_print --preview --viewall --autocenter --imgsize=1024,768 -o "$OutDir\preview_lid_print.png" "$ScadFile"
& $OpenScad -D mode=print --preview --viewall --autocenter --imgsize=1280,720 -o "$OutDir\preview_print.png" "$ScadFile"

Write-Host "=== All files successfully generated in $OutDir ===" -ForegroundColor Green
Get-ChildItem -Path "$OutDir" -Include *.stl,*.3mf,*.png,*.scad | Select-Object Name, Length, LastWriteTime
