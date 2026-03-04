import AVFoundation
import CoreVideo
import Foundation

// Thread-safe frame buffer fed by AVCaptureSession delegate callbacks.
// Uses a pre-allocated UnsafeMutableBufferPointer to avoid per-frame [UInt8] allocations.
final class FrameGrabber: NSObject, AVCaptureVideoDataOutputSampleBufferDelegate {
    private let lock = NSLock()
    private var buffer: UnsafeMutablePointer<UInt8>? = nil
    private var bufferCapacity: Int = 0
    private var frameSize: Int = 0
    private var frameWidth: Int32 = 0
    private var frameHeight: Int32 = 0

    deinit {
        buffer?.deallocate()
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didOutput sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        guard let pb = CMSampleBufferGetImageBuffer(sampleBuffer) else { return }
        CVPixelBufferLockBaseAddress(pb, .readOnly)
        defer { CVPixelBufferUnlockBaseAddress(pb, .readOnly) }

        let w = CVPixelBufferGetWidth(pb)
        let h = CVPixelBufferGetHeight(pb)
        let stride = CVPixelBufferGetBytesPerRow(pb)
        guard let base = CVPixelBufferGetBaseAddress(pb) else { return }

        let rowBytes = w * 4
        let needed = w * h * 4

        lock.lock()

        // Grow the internal buffer if needed (rare — only on resolution change).
        if needed > bufferCapacity {
            buffer?.deallocate()
            buffer = .allocate(capacity: needed)
            bufferCapacity = needed
        }

        guard let dst = buffer else {
            lock.unlock()
            return
        }

        // Copy row-by-row because stride may exceed width*4 due to GPU padding.
        for row in 0..<h {
            memcpy(dst.advanced(by: row * rowBytes),
                   base.advanced(by: row * stride), rowBytes)
        }
        frameSize = needed
        frameWidth = Int32(w)
        frameHeight = Int32(h)

        lock.unlock()
    }

    /// Copy latest frame into caller's buffer under the lock (single atomic operation).
    /// Returns (neededBytes, width, height). neededBytes == 0 means no frame yet.
    func copyFrame(into buf: UnsafeMutablePointer<UInt8>?, bufLen: Int) -> (Int, Int32, Int32) {
        lock.lock()
        defer { lock.unlock() }

        if frameSize == 0 { return (0, 0, 0) }

        if let buf = buf, bufLen >= frameSize, let src = buffer {
            memcpy(buf, src, frameSize)
        }
        return (frameSize, frameWidth, frameHeight)
    }
}

final class CameraBridge {
    let session = AVCaptureSession()
    let delegate = FrameGrabber()

    func start() -> Bool {
        guard ensureCameraAccess() else { return false }
        guard let device = AVCaptureDevice.default(for: .video),
              let input = try? AVCaptureDeviceInput(device: device) else {
            logErr("camera_init: no usable video device\n")
            return false
        }

        let output = AVCaptureVideoDataOutput()
        output.videoSettings = [
            kCVPixelBufferPixelFormatTypeKey as String: Int(kCVPixelFormatType_32BGRA)
        ]
        output.alwaysDiscardsLateVideoFrames = true
        output.setSampleBufferDelegate(delegate, queue: DispatchQueue(label: "camera.frame"))

        if session.canSetSessionPreset(.vga640x480) { session.sessionPreset = .vga640x480 }
        guard session.canAddInput(input), session.canAddOutput(output) else {
            logErr("camera_init: cannot configure capture session\n")
            return false
        }

        session.addInput(input)
        session.addOutput(output)
        session.startRunning()
        return session.isRunning
    }

    func stop() { session.stopRunning() }
}

// MARK: - Global state (protected by gLock for thread safety)

private let gLock = NSLock()
private var gCamera: CameraBridge?

// MARK: - C API exposed to Rust via FFI

/// Returns 0 on success, -1 on failure.
@_cdecl("camera_init")
public func camera_init() -> Int32 {
    gLock.lock()
    gCamera?.stop()
    gCamera = nil
    gLock.unlock()

    let cam = CameraBridge()
    guard cam.start() else { return -1 }

    gLock.lock()
    gCamera = cam
    gLock.unlock()
    return 0
}

/// Returns: >0 = bytes needed (copied if buf large enough), 0 = no frame yet, <0 = error.
@_cdecl("camera_get_frame")
public func camera_get_frame(
    _ buf: UnsafeMutablePointer<UInt8>?,
    _ bufLen: Int32,
    _ outWidth: UnsafeMutablePointer<Int32>?,
    _ outHeight: UnsafeMutablePointer<Int32>?
) -> Int32 {
    gLock.lock()
    let cam = gCamera
    gLock.unlock()
    guard let cam else { return -1 }

    let (needed, w, h) = cam.delegate.copyFrame(into: buf, bufLen: Int(bufLen))
    outWidth?.pointee = w
    outHeight?.pointee = h

    if needed == 0 { return 0 }
    guard needed <= Int(Int32.max) else { return -1 }
    return Int32(needed)
}

@_cdecl("camera_stop")
public func camera_stop() {
    gLock.lock()
    gCamera?.stop()
    gCamera = nil
    gLock.unlock()
}

// MARK: - Helpers

private func logErr(_ msg: String) {
    msg.withCString { fputs($0, stderr) }
}

private func ensureCameraAccess() -> Bool {
    switch AVCaptureDevice.authorizationStatus(for: .video) {
    case .authorized: return true
    case .notDetermined:
        AVCaptureDevice.requestAccess(for: .video) { _ in }
        logErr("camera_init: permission prompt shown; re-run after granting\n")
        return false
    case .denied, .restricted:
        logErr("camera_init: camera access denied\n")
        return false
    @unknown default: return false
    }
}
