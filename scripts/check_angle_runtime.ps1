[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$RuntimeDir
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-ProbeEvent {
    param(
        [Parameter(Mandatory = $true)][string]$Event,
        [Parameter(Mandatory = $true)][string]$State,
        [hashtable]$Fields = @{}
    )

    $record = [ordered]@{
        event = $Event
        state = $State
    }
    foreach ($entry in $Fields.GetEnumerator()) {
        $record[$entry.Key] = $entry.Value
    }
    Write-Output ($record | ConvertTo-Json -Compress)
}

try {
    $resolvedRuntime = (Resolve-Path -LiteralPath $RuntimeDir).Path
    Write-ProbeEvent -Event "angle.runtime_probe.started" -State "started" -Fields @{
        runtime = $resolvedRuntime
    }

    foreach ($name in @("libEGL.dll", "libGLESv2.dll", "vulkan-1.dll")) {
        $path = Join-Path $resolvedRuntime $name
        $item = Get-Item -LiteralPath $path -ErrorAction Stop
        if (-not $item.PSIsContainer -and $item.Length -gt 0) {
            continue
        }
        throw "ANGLE runtime file is empty or not a regular file: $path"
    }

    $env:PATH = "$resolvedRuntime;$env:PATH"
    if (-not ("AngleEglRuntimeProbe" -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;

public static class AngleEglRuntimeProbe
{
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr LoadLibraryW(string path);

    [DllImport("kernel32.dll", CharSet = CharSet.Ansi, SetLastError = true)]
    private static extern IntPtr GetProcAddress(IntPtr module, string name);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool FreeLibrary(IntPtr module);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate IntPtr EglGetDisplay(IntPtr nativeDisplay);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate uint EglInitialize(IntPtr display, out int major, out int minor);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate uint EglGetError();

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate uint EglTerminate(IntPtr display);

    public sealed class Result
    {
        public uint Initialized;
        public int Major;
        public int Minor;
        public uint Error;
    }

    private static T Resolve<T>(IntPtr module, string name) where T : class
    {
        IntPtr address = GetProcAddress(module, name);
        if (address == IntPtr.Zero)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Missing EGL export " + name);
        }
        return Marshal.GetDelegateForFunctionPointer(address, typeof(T)) as T;
    }

    public static Result Run(string eglPath)
    {
        IntPtr module = LoadLibraryW(eglPath);
        if (module == IntPtr.Zero)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "LoadLibraryW(libEGL.dll)");
        }

        try
        {
            EglGetDisplay getDisplay = Resolve<EglGetDisplay>(module, "eglGetDisplay");
            EglInitialize initialize = Resolve<EglInitialize>(module, "eglInitialize");
            EglGetError getError = Resolve<EglGetError>(module, "eglGetError");
            EglTerminate terminate = Resolve<EglTerminate>(module, "eglTerminate");
            IntPtr display = getDisplay(IntPtr.Zero);
            int major;
            int minor;
            uint initialized = initialize(display, out major, out minor);
            uint error = getError();
            if (initialized != 0)
            {
                terminate(display);
            }
            return new Result {
                Initialized = initialized,
                Major = major,
                Minor = minor,
                Error = error,
            };
        }
        finally
        {
            FreeLibrary(module);
        }
    }
}
'@
    }

    $eglPath = Join-Path $resolvedRuntime "libEGL.dll"
    $result = [AngleEglRuntimeProbe]::Run($eglPath)
    if ($result.Initialized -eq 0) {
        throw ("eglInitialize failed with EGL error 0x{0:X4}" -f $result.Error)
    }
    if ($result.Major -lt 1 -or ($result.Major -eq 1 -and $result.Minor -lt 5)) {
        throw "ANGLE initialized an unexpected EGL version $($result.Major).$($result.Minor)"
    }

    Write-ProbeEvent -Event "angle.runtime_probe.succeeded" -State "succeeded" -Fields @{
        egl_version = "$($result.Major).$($result.Minor)"
        egl_error = ("0x{0:X4}" -f $result.Error)
        vulkan_loader = "bundled"
    }
} catch {
    Write-ProbeEvent -Event "angle.runtime_probe.failed" -State "failed" -Fields @{
        code = "angle_runtime_initialization_failed"
        detail = $_.Exception.Message
    }
    exit 1
}
