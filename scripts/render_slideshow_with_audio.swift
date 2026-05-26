import Foundation
import AVFoundation
import AppKit

func fail(_ message: String) -> Never {
    fputs("ERROR: \(message)\n", stderr)
    exit(1)
}

if CommandLine.arguments.count != 4 {
    fail("usage: swift render_slideshow_with_audio.swift <slides-dir> <audio-aiff> <output-mp4>")
}

let slidesDir = URL(fileURLWithPath: CommandLine.arguments[1])
let audioURL = URL(fileURLWithPath: CommandLine.arguments[2])
let outputURL = URL(fileURLWithPath: CommandLine.arguments[3])
let tempVideoURL = outputURL.deletingLastPathComponent().appendingPathComponent("lp0002-demo-video-only.mov")

let fm = FileManager.default
try? fm.removeItem(at: outputURL)
try? fm.removeItem(at: tempVideoURL)

let slideURLs = (try? fm.contentsOfDirectory(at: slidesDir, includingPropertiesForKeys: nil))?
    .filter { $0.pathExtension.lowercased() == "png" }
    .sorted { $0.lastPathComponent < $1.lastPathComponent } ?? []
if slideURLs.isEmpty { fail("no PNG slides in \(slidesDir.path)") }

let width = 1280
let height = 720
let fps: Int32 = 24
let audioAsset = AVURLAsset(url: audioURL)
let audioDuration = try await audioAsset.load(.duration)
let totalSeconds = max(CMTimeGetSeconds(audioDuration), Double(slideURLs.count * 6))
let totalFrames = Int(ceil(totalSeconds * Double(fps)))

func makePixelBuffer(from image: NSImage) -> CVPixelBuffer? {
    var pixelBuffer: CVPixelBuffer?
    let attrs: [String: Any] = [
        kCVPixelBufferCGImageCompatibilityKey as String: true,
        kCVPixelBufferCGBitmapContextCompatibilityKey as String: true,
        kCVPixelBufferWidthKey as String: width,
        kCVPixelBufferHeightKey as String: height,
    ]
    let status = CVPixelBufferCreate(kCFAllocatorDefault, width, height, kCVPixelFormatType_32ARGB, attrs as CFDictionary, &pixelBuffer)
    guard status == kCVReturnSuccess, let pb = pixelBuffer else { return nil }
    CVPixelBufferLockBaseAddress(pb, [])
    defer { CVPixelBufferUnlockBaseAddress(pb, []) }
    guard let ctx = CGContext(
        data: CVPixelBufferGetBaseAddress(pb),
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: CVPixelBufferGetBytesPerRow(pb),
        space: CGColorSpaceCreateDeviceRGB(),
        bitmapInfo: CGImageAlphaInfo.noneSkipFirst.rawValue
    ) else { return nil }
    ctx.setFillColor(NSColor.black.cgColor)
    ctx.fill(CGRect(x: 0, y: 0, width: width, height: height))
    guard let cg = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else { return nil }
    ctx.draw(cg, in: CGRect(x: 0, y: 0, width: width, height: height))
    return pb
}

let writer = try AVAssetWriter(outputURL: tempVideoURL, fileType: .mov)
let videoSettings: [String: Any] = [
    AVVideoCodecKey: AVVideoCodecType.h264,
    AVVideoWidthKey: width,
    AVVideoHeightKey: height,
    AVVideoCompressionPropertiesKey: [
        AVVideoAverageBitRateKey: 1_200_000,
        AVVideoProfileLevelKey: AVVideoProfileLevelH264HighAutoLevel,
    ],
]
let input = AVAssetWriterInput(mediaType: .video, outputSettings: videoSettings)
input.expectsMediaDataInRealTime = false
let adaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: input, sourcePixelBufferAttributes: [
    kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32ARGB,
    kCVPixelBufferWidthKey as String: width,
    kCVPixelBufferHeightKey as String: height,
])
if !writer.canAdd(input) { fail("cannot add video input") }
writer.add(input)
if !writer.startWriting() { fail("writer failed to start: \(writer.error?.localizedDescription ?? "unknown")") }
writer.startSession(atSourceTime: .zero)

let slideBuffers = slideURLs.map { url -> CVPixelBuffer in
    guard let img = NSImage(contentsOf: url), let pb = makePixelBuffer(from: img) else {
        fail("failed to load/render slide \(url.path)")
    }
    return pb
}

var frame = 0
let frameDuration = CMTime(value: 1, timescale: fps)
while frame < totalFrames {
    while !input.isReadyForMoreMediaData { Thread.sleep(forTimeInterval: 0.01) }
    let progress = Double(frame) / Double(max(totalFrames - 1, 1))
    let slideIndex = min(Int(progress * Double(slideBuffers.count)), slideBuffers.count - 1)
    let time = CMTimeMultiply(frameDuration, multiplier: Int32(frame))
    if !adaptor.append(slideBuffers[slideIndex], withPresentationTime: time) {
        fail("failed to append frame \(frame): \(writer.error?.localizedDescription ?? "unknown")")
    }
    frame += 1
}
input.markAsFinished()
await writer.finishWriting()
if writer.status != .completed { fail("video writer did not complete: \(writer.error?.localizedDescription ?? "unknown")") }

let composition = AVMutableComposition()
let videoAsset = AVURLAsset(url: tempVideoURL)
let videoTracks = try await videoAsset.loadTracks(withMediaType: .video)
guard let videoTrack = videoTracks.first else { fail("temporary video has no video track") }
let range = CMTimeRange(start: .zero, duration: try await videoAsset.load(.duration))
let compVideo = composition.addMutableTrack(withMediaType: .video, preferredTrackID: kCMPersistentTrackID_Invalid)
try compVideo?.insertTimeRange(range, of: videoTrack, at: .zero)

let audioTracks = try await audioAsset.loadTracks(withMediaType: .audio)
if let audioTrack = audioTracks.first {
    let compAudio = composition.addMutableTrack(withMediaType: .audio, preferredTrackID: kCMPersistentTrackID_Invalid)
    try compAudio?.insertTimeRange(CMTimeRange(start: .zero, duration: audioDuration), of: audioTrack, at: .zero)
}

guard let exporter = AVAssetExportSession(asset: composition, presetName: AVAssetExportPreset1280x720) else {
    fail("cannot create exporter")
}
exporter.outputURL = outputURL
exporter.outputFileType = .mp4
try await exporter.export(to: outputURL, as: .mp4)
if exporter.status != .completed { fail("export failed: \(exporter.error?.localizedDescription ?? "unknown")") }
print("rendered \(outputURL.path)")
