' Embroidery Catalogue — Silent Launcher (Release)
' Launches the release .exe without any console window.
' Double-click this file or call it from a shortcut.

Dim shell, fso, scriptDir
Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
scriptDir = fso.GetParentFolderName(WScript.ScriptFullName)

shell.Run """" & scriptDir & "\target\release\embroidery-catalogue.exe""", 0, False