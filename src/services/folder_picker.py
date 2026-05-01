"""Helpers for opening native folder picker dialogs.

This module keeps OS-specific picker code isolated from FastAPI routes so the
import workflow can use the best available folder selection UI on Windows while
still falling back gracefully on other systems.
"""

from __future__ import annotations

import ctypes
import logging
import os
from ctypes import POINTER, Structure, byref, c_long, c_ulong, c_void_p, c_wchar_p, cast
from ctypes.wintypes import DWORD, HWND, LPCWSTR, LPWSTR, WORD
from uuid import UUID

from src.config import native_dialogs_disabled

log = logging.getLogger(__name__)
WINFUNCTYPE = getattr(ctypes, "WINFUNCTYPE", ctypes.CFUNCTYPE)


class FolderPickerUnavailableError(RuntimeError):
    """Raised when no folder picker implementation can be opened."""


def display_path(path: str) -> str:
    """Return a Windows-friendly display path using backslashes."""
    if not path:
        return ""
    return os.path.normpath(path).replace("/", "\\")


def resolve_picker_initial_dir(start_dir: str, *, prefer_parent: bool = False) -> str:
    """Return a safe starting directory for the native folder picker.

    When ``prefer_parent`` is true and *start_dir* is an existing folder, the
    parent of that folder is returned where possible. This fits folder-picking
    workflows where the user previously navigated to a parent and then selected
    one child folder for import.
    """
    initial = (start_dir or "").strip()
    if os.path.isabs(initial):
        if os.path.isdir(initial):
            if prefer_parent:
                parent = os.path.dirname(initial.rstrip("\\/"))
                if parent and os.path.isdir(parent):
                    return parent
            return initial
        parent = os.path.dirname(initial.rstrip("\\/"))
        if parent and os.path.isdir(parent):
            return parent

    initial = os.path.expanduser("~")
    if not os.path.isdir(initial):
        initial = "C:\\"
    return initial


def pick_folder(start_dir: str = "", title: str = "Select folder") -> str:
    """Open a native picker for a single folder and return the chosen path."""
    paths = pick_folders(start_dir=start_dir, title=title, allow_multiple=False)
    return paths[0] if paths else ""


def pick_folders(
    start_dir: str = "",
    title: str = "Select folder(s)",
    allow_multiple: bool = True,
) -> list[str]:
    """Open the best available folder picker and return the selected paths.

    On Windows this prefers the native Common Item Dialog with true multi-select.
    If that is unavailable, it falls back to ``tkinter.askdirectory()``, which is
    still useful as a single-folder fallback.
    """
    initial = resolve_picker_initial_dir(start_dir)

    if native_dialogs_disabled():
        raise FolderPickerUnavailableError(
            "Native file dialogs are disabled by the current environment. Please enter the path manually."
        )

    if os.name == "nt":
        try:
            return _pick_folders_windows(initial, title=title, allow_multiple=allow_multiple)
        except Exception as exc:  # noqa: BLE001
            log.warning(
                "Native Windows folder picker unavailable; falling back to tkinter: %s", exc
            )

    try:
        import tkinter as tk
        from tkinter import filedialog

        root = tk.Tk()
        root.withdraw()
        root.wm_attributes("-topmost", True)
        path = filedialog.askdirectory(title=title, initialdir=initial)
        root.destroy()
        return [display_path(path)] if path else []
    except Exception as exc:  # noqa: BLE001
        raise FolderPickerUnavailableError(str(exc)) from exc


# ---------------------------------------------------------------------------
# Windows native Common Item Dialog implementation (IFileOpenDialog)
# ---------------------------------------------------------------------------

HRESULT = c_long
BYTE = ctypes.c_ubyte
CLSCTX_INPROC_SERVER = 0x1
COINIT_APARTMENTTHREADED = 0x2
COINIT_DISABLE_OLE1DDE = 0x4
SIGDN_FILESYSPATH = 0x80058000
FOS_PICKFOLDERS = 0x20
FOS_FORCEFILESYSTEM = 0x40
FOS_ALLOWMULTISELECT = 0x200
FOS_PATHMUSTEXIST = 0x800
ERROR_CANCELLED = 0x800704C7


class GUID(Structure):
    _fields_ = [
        ("Data1", DWORD),
        ("Data2", WORD),
        ("Data3", WORD),
        ("Data4", BYTE * 8),
    ]

    def __init__(self, value: str) -> None:
        super().__init__()
        raw = UUID(value).bytes_le
        self.Data1 = int.from_bytes(raw[0:4], "little")
        self.Data2 = int.from_bytes(raw[4:6], "little")
        self.Data3 = int.from_bytes(raw[6:8], "little")
        self.Data4[:] = raw[8:16]


class IFileOpenDialogVtbl(Structure):
    _fields_ = [
        ("QueryInterface", WINFUNCTYPE(HRESULT, c_void_p, POINTER(GUID), POINTER(c_void_p))),
        ("AddRef", WINFUNCTYPE(c_ulong, c_void_p)),
        ("Release", WINFUNCTYPE(c_ulong, c_void_p)),
        ("Show", WINFUNCTYPE(HRESULT, c_void_p, HWND)),
        ("SetFileTypes", c_void_p),
        ("SetFileTypeIndex", c_void_p),
        ("GetFileTypeIndex", c_void_p),
        ("Advise", c_void_p),
        ("Unadvise", c_void_p),
        ("SetOptions", WINFUNCTYPE(HRESULT, c_void_p, DWORD)),
        ("GetOptions", WINFUNCTYPE(HRESULT, c_void_p, POINTER(DWORD))),
        ("SetDefaultFolder", WINFUNCTYPE(HRESULT, c_void_p, c_void_p)),
        ("SetFolder", WINFUNCTYPE(HRESULT, c_void_p, c_void_p)),
        ("GetFolder", c_void_p),
        ("GetCurrentSelection", c_void_p),
        ("SetFileName", c_void_p),
        ("GetFileName", c_void_p),
        ("SetTitle", WINFUNCTYPE(HRESULT, c_void_p, LPCWSTR)),
        ("SetOkButtonLabel", c_void_p),
        ("SetFileNameLabel", c_void_p),
        ("GetResult", WINFUNCTYPE(HRESULT, c_void_p, POINTER(c_void_p))),
        ("AddPlace", c_void_p),
        ("SetDefaultExtension", c_void_p),
        ("Close", c_void_p),
        ("SetClientGuid", c_void_p),
        ("ClearClientData", c_void_p),
        ("SetFilter", c_void_p),
        ("GetResults", WINFUNCTYPE(HRESULT, c_void_p, POINTER(c_void_p))),
        ("GetSelectedItems", c_void_p),
    ]


class IFileOpenDialog(Structure):
    _fields_ = [("lpVtbl", POINTER(IFileOpenDialogVtbl))]


class IShellItemVtbl(Structure):
    _fields_ = [
        ("QueryInterface", WINFUNCTYPE(HRESULT, c_void_p, POINTER(GUID), POINTER(c_void_p))),
        ("AddRef", WINFUNCTYPE(c_ulong, c_void_p)),
        ("Release", WINFUNCTYPE(c_ulong, c_void_p)),
        ("BindToHandler", c_void_p),
        ("GetParent", c_void_p),
        ("GetDisplayName", WINFUNCTYPE(HRESULT, c_void_p, DWORD, POINTER(LPWSTR))),
        ("GetAttributes", c_void_p),
        ("Compare", c_void_p),
    ]


class IShellItem(Structure):
    _fields_ = [("lpVtbl", POINTER(IShellItemVtbl))]


class IShellItemArrayVtbl(Structure):
    _fields_ = [
        ("QueryInterface", WINFUNCTYPE(HRESULT, c_void_p, POINTER(GUID), POINTER(c_void_p))),
        ("AddRef", WINFUNCTYPE(c_ulong, c_void_p)),
        ("Release", WINFUNCTYPE(c_ulong, c_void_p)),
        ("BindToHandler", c_void_p),
        ("GetPropertyStore", c_void_p),
        ("GetPropertyDescriptionList", c_void_p),
        ("GetAttributes", c_void_p),
        ("GetCount", WINFUNCTYPE(HRESULT, c_void_p, POINTER(DWORD))),
        ("GetItemAt", WINFUNCTYPE(HRESULT, c_void_p, DWORD, POINTER(c_void_p))),
        ("EnumItems", c_void_p),
    ]


class IShellItemArray(Structure):
    _fields_ = [("lpVtbl", POINTER(IShellItemArrayVtbl))]


CLSID_FileOpenDialog = GUID("DC1C5A9C-E88A-4DDE-A5A1-60F82A20AEF7")
IID_IFileOpenDialog = GUID("D57C7288-D4AD-4768-BE02-9D969532D960")
IID_IShellItem = GUID("43826D1E-E718-42EE-BC55-A1E261C37BFE")

if os.name == "nt":
    ole32 = ctypes.OleDLL("ole32")
    shell32 = ctypes.OleDLL("shell32")
    user32 = ctypes.WinDLL("user32", use_last_error=True)

    ole32.CoInitializeEx.argtypes = [c_void_p, DWORD]
    ole32.CoInitializeEx.restype = HRESULT
    ole32.CoUninitialize.argtypes = []
    ole32.CoUninitialize.restype = None
    ole32.CoCreateInstance.argtypes = [
        POINTER(GUID),
        c_void_p,
        DWORD,
        POINTER(GUID),
        POINTER(c_void_p),
    ]
    ole32.CoCreateInstance.restype = HRESULT
    ole32.CoTaskMemFree.argtypes = [c_void_p]
    ole32.CoTaskMemFree.restype = None
    shell32.SHCreateItemFromParsingName.argtypes = [
        LPCWSTR,
        c_void_p,
        POINTER(GUID),
        POINTER(c_void_p),
    ]
    shell32.SHCreateItemFromParsingName.restype = HRESULT
    user32.GetForegroundWindow.argtypes = []
    user32.GetForegroundWindow.restype = HWND
    user32.GetActiveWindow.argtypes = []
    user32.GetActiveWindow.restype = HWND
    user32.SetForegroundWindow.argtypes = [HWND]
    user32.SetForegroundWindow.restype = ctypes.c_bool
    user32.BringWindowToTop.argtypes = [HWND]
    user32.BringWindowToTop.restype = ctypes.c_bool
else:  # pragma: no cover - only used on Windows
    ole32 = None
    shell32 = None
    user32 = None


def _check_hresult(hr: int, *, cancel_allowed: bool = False) -> None:
    value = c_ulong(hr).value
    if cancel_allowed and value == ERROR_CANCELLED:
        raise _DialogCancelled
    if hr < 0:
        raise OSError(f"HRESULT 0x{value:08X}")


class _DialogCancelled(Exception):
    pass


def _release(ptr: c_void_p | POINTER(Structure) | None) -> None:
    if not ptr:
        return
    try:
        ptr.contents.lpVtbl.contents.Release(ptr)  # type: ignore[union-attr]
    except Exception:  # noqa: BLE001
        pass


def _shell_item_path(item: POINTER(IShellItem)) -> str:
    raw_path = LPWSTR()
    _check_hresult(
        item.contents.lpVtbl.contents.GetDisplayName(item, SIGDN_FILESYSPATH, byref(raw_path))
    )
    try:
        return display_path(raw_path.value or "")
    finally:
        if raw_path and ole32 is not None:
            ole32.CoTaskMemFree(cast(raw_path, c_void_p))


def _pick_folders_windows(initial_dir: str, title: str, allow_multiple: bool) -> list[str]:
    """Use the Windows Common Item Dialog to pick one or more folders."""
    dialog_ptr = c_void_p()
    default_folder_ptr = c_void_p()
    initialized = False
    owner_hwnd = None

    try:
        hr = ole32.CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
        if hr not in (0, 1):
            _check_hresult(hr)
        initialized = True

        _check_hresult(
            ole32.CoCreateInstance(
                byref(CLSID_FileOpenDialog),
                None,
                CLSCTX_INPROC_SERVER,
                byref(IID_IFileOpenDialog),
                byref(dialog_ptr),
            )
        )
        dialog = cast(dialog_ptr, POINTER(IFileOpenDialog))

        options = DWORD()
        _check_hresult(dialog.contents.lpVtbl.contents.GetOptions(dialog, byref(options)))
        new_options = options.value | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST
        if allow_multiple:
            new_options |= FOS_ALLOWMULTISELECT
        _check_hresult(dialog.contents.lpVtbl.contents.SetOptions(dialog, new_options))

        if title:
            _check_hresult(dialog.contents.lpVtbl.contents.SetTitle(dialog, c_wchar_p(title)))

        if initial_dir and os.path.isdir(initial_dir):
            hr = shell32.SHCreateItemFromParsingName(
                c_wchar_p(initial_dir),
                None,
                byref(IID_IShellItem),
                byref(default_folder_ptr),
            )
            if hr >= 0 and default_folder_ptr:
                default_folder = cast(default_folder_ptr, POINTER(IShellItem))
                try:
                    dialog.contents.lpVtbl.contents.SetFolder(dialog, default_folder)
                except Exception:  # noqa: BLE001
                    pass
                try:
                    dialog.contents.lpVtbl.contents.SetDefaultFolder(dialog, default_folder)
                except Exception:  # noqa: BLE001
                    pass

        if user32 is not None:
            owner_hwnd = user32.GetForegroundWindow() or user32.GetActiveWindow()
            if owner_hwnd:
                try:
                    user32.BringWindowToTop(owner_hwnd)
                    user32.SetForegroundWindow(owner_hwnd)
                except Exception:  # noqa: BLE001
                    pass

        _check_hresult(
            dialog.contents.lpVtbl.contents.Show(dialog, owner_hwnd), cancel_allowed=True
        )

        if allow_multiple:
            results_ptr = c_void_p()
            _check_hresult(dialog.contents.lpVtbl.contents.GetResults(dialog, byref(results_ptr)))
            results = cast(results_ptr, POINTER(IShellItemArray))
            try:
                count = DWORD()
                _check_hresult(results.contents.lpVtbl.contents.GetCount(results, byref(count)))
                paths: list[str] = []
                for index in range(count.value):
                    item_ptr = c_void_p()
                    _check_hresult(
                        results.contents.lpVtbl.contents.GetItemAt(results, index, byref(item_ptr))
                    )
                    item = cast(item_ptr, POINTER(IShellItem))
                    try:
                        path = _shell_item_path(item)
                        if path:
                            paths.append(path)
                    finally:
                        _release(item)
                return paths
            finally:
                _release(results)

        result_ptr = c_void_p()
        _check_hresult(dialog.contents.lpVtbl.contents.GetResult(dialog, byref(result_ptr)))
        result_item = cast(result_ptr, POINTER(IShellItem))
        try:
            path = _shell_item_path(result_item)
            return [path] if path else []
        finally:
            _release(result_item)

    except _DialogCancelled:
        return []
    finally:
        if default_folder_ptr:
            _release(cast(default_folder_ptr, POINTER(IShellItem)))
        if dialog_ptr:
            _release(cast(dialog_ptr, POINTER(IFileOpenDialog)))
        if initialized:
            ole32.CoUninitialize()
