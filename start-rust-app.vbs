' Embroidery Catalogue — Silent Dev Launcher
' Runs start-rust-app.bat behind the scenes without showing its console window.
' Double-click this file to launch the app in dev mode.
' Ensure start-rust-app.bat exists in the same folder.

Dim shell, fso, scriptDir
Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
scriptDir = fso.GetParentFolderName(WScript.ScriptFullName)

shell.Run """" & scriptDir & "\start-rust-app.bat""", 0, False