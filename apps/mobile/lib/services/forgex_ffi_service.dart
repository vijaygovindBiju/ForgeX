import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';

typedef ForgexFfiOpenC = Pointer<Void> Function(Pointer<Utf8> dbPath);
typedef ForgexFfiOpenDart = Pointer<Void> Function(Pointer<Utf8> dbPath);

typedef ForgexFfiCloseC = Void Function(Pointer<Void> ctx);
typedef ForgexFfiCloseDart = void Function(Pointer<Void> ctx);

typedef ForgexFfiStatusC = Pointer<Utf8> Function(Pointer<Void> ctx);
typedef ForgexFfiStatusDart = Pointer<Utf8> Function(Pointer<Void> ctx);

typedef ForgexFfiFreeStringC = Void Function(Pointer<Utf8> str);
typedef ForgexFfiFreeStringDart = void Function(Pointer<Utf8> str);

class ForgeXFfiService {
  DynamicLibrary? _lib;
  Pointer<Void>? _ctx;
  bool _isAvailable = false;

  bool get isAvailable => _isAvailable;

  void init({String? dbPath}) {
    try {
      if (Platform.isAndroid) {
        _lib = DynamicLibrary.open('libforgex_ffi.so');
      } else if (Platform.isLinux) {
        _lib = DynamicLibrary.open('libforgex_ffi.so');
      } else {
        _lib = DynamicLibrary.process();
      }

      final openFn = _lib!.lookupFunction<ForgexFfiOpenC, ForgexFfiOpenDart>('forgex_ffi_open');
      if (dbPath != null && dbPath.isNotEmpty) {
        final pathPtr = dbPath.toNativeUtf8();
        _ctx = openFn(pathPtr);
        calloc.free(pathPtr);
      } else {
        _ctx = openFn(nullptr);
      }

      _isAvailable = _ctx != null && _ctx != nullptr;
    } catch (_) {
      _isAvailable = false;
    }
  }

  Map<String, dynamic>? getStatus() {
    if (!_isAvailable || _ctx == null) return null;
    try {
      final statusFn = _lib!.lookupFunction<ForgexFfiStatusC, ForgexFfiStatusDart>('forgex_ffi_status');
      final freeFn = _lib!.lookupFunction<ForgexFfiFreeStringC, ForgexFfiFreeStringDart>('forgex_ffi_free_string');

      final ptr = statusFn(_ctx!);
      if (ptr == nullptr) return null;

      final jsonStr = ptr.toDartString();
      freeFn(ptr);

      return jsonDecode(jsonStr);
    } catch (_) {
      return null;
    }
  }

  void dispose() {
    if (_isAvailable && _ctx != null) {
      final closeFn = _lib!.lookupFunction<ForgexFfiCloseC, ForgexFfiCloseDart>('forgex_ffi_close');
      closeFn(_ctx!);
      _ctx = null;
      _isAvailable = false;
    }
  }
}
