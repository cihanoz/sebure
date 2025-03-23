import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

/// Service for QR code generation and scanning
class QrService {
  // Singleton instance
  static final QrService _instance = QrService._internal();
  static QrService get instance => _instance;

  // Private constructor for singleton
  QrService._internal();

  /// Generate a QR code widget for an address
  Widget generateQrCode({
    required String data,
    double size = 200.0,
    Color backgroundColor = Colors.white,
    Color foregroundColor = Colors.black,
  }) {
    return QrImageView(
      data: data,
      version: QrVersions.auto,
      size: size,
      backgroundColor: backgroundColor,
      foregroundColor: foregroundColor,
      errorStateBuilder: (context, error) {
        return Container(
          width: size,
          height: size,
          color: backgroundColor,
          child: Center(
            child: Text(
              'Error generating QR code',
              style: TextStyle(color: foregroundColor),
              textAlign: TextAlign.center,
            ),
          ),
        );
      },
    );
  }

  /// Scan a QR code from the camera
  Widget buildQrScanner({
    required Function(String) onDetect,
    required Function() onCancel,
  }) {
    return MobileScanner(
      controller: MobileScannerController(
        detectionSpeed: DetectionSpeed.normal,
        facing: CameraFacing.back,
      ),
      onDetect: (capture) {
        final List<Barcode> barcodes = capture.barcodes;
        if (barcodes.isNotEmpty && barcodes[0].rawValue != null) {
          final String code = barcodes[0].rawValue!;
          onDetect(code);
        }
      },
    );
  }

  /// Scan a QR code from an image in the gallery
  Future<String?> scanQrFromGallery() async {
    try {
      final ImagePicker picker = ImagePicker();
      final XFile? image = await picker.pickImage(source: ImageSource.gallery);

      if (image == null) return null;

      // This is a placeholder - in a real implementation, you would use
      // a library that can decode QR codes from images
      // For now, we'll just return a mock result
      return 'mock_address_from_qr_code';
    } catch (e) {
      debugPrint('Error scanning QR from gallery: $e');
      return null;
    }
  }

  /// Validate if a string is a valid blockchain address
  bool isValidAddress(String address) {
    // This is a placeholder - in a real implementation, you would
    // validate the address format according to your blockchain's rules
    // For now, we'll just check if it's a non-empty string with a minimum length
    return address.isNotEmpty && address.length >= 20;
  }
}
