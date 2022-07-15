// ignore_for_file: always_declare_return_types

import 'dart:io';
import 'package:effektio/common/widget/AppCommon.dart';
import 'package:effektio_flutter_sdk/effektio_flutter_sdk_ffi.dart'
    show
        Conversation,
        FileDescription,
        ImageDescription,
        RoomMessage,
        TimelineStream;
import 'package:file_picker/file_picker.dart';
import 'package:filesystem_picker/filesystem_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_chat_types/flutter_chat_types.dart' as types;
import 'package:get/get.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mime/mime.dart';
import 'package:mutex/mutex.dart';
import 'package:open_file/open_file.dart';
import 'package:path_provider/path_provider.dart';
import 'package:permission_handler/permission_handler.dart';

final mtx = Mutex(); // acquire this mutex, when updating state

class ChatController extends GetxController {
  List<types.Message> messages = [];
  TimelineStream? _stream;
  RxBool isLoading = false.obs;
  int _page = 0;
  late final Conversation room;
  late final types.User user;
  final bool _isDesktop = !(Platform.isAndroid || Platform.isIOS);

  //get the timeline of room
  init(Conversation convoRoom, types.User convoUser) async {
    room = convoRoom;
    user = convoUser;
    await _fetchTimeline();
  }

  Future<void> _fetchTimeline() async {
    isLoading.value = true;
    _stream = await room.timeline();
    // i am fetching messages from remote
    var _messages = await _stream!.paginateBackwards(10);
    mtx.acquire();
    for (RoomMessage message in _messages) {
      _loadMessage(message, messages);
    }
    isLoading.value = false;
    if (messages.isNotEmpty) {
      if (messages.first.author.id == user.id) {
        bool isSeen = await room.readReceipt(messages.first.id);
        messages[0] = messages[0].copyWith(
          showStatus: true,
          status: isSeen ? types.Status.seen : types.Status.delivered,
        );
      }
    }
    mtx.release();
  }

  //waits for new event
  Future<void> newEvent() async {
    await _stream!.next();
    var message = await room.latestMessage();
    _loadMessage(message, messages);
    update(['Chat']);
  }

  //preview message link
  void handlePreviewDataFetched(
    types.TextMessage message,
    types.PreviewData previewData,
  ) {
    final index = messages.indexWhere((element) => element.id == message.id);
    final updatedMessage = messages[index].copyWith(previewData: previewData);

    WidgetsBinding.instance.addPostFrameCallback((_) {
      messages[index] = updatedMessage;
      update(['Chat']);
    });
  }

  //push messages in conversation
  Future<void> handleSendPressed(types.PartialText message) async {
    // image or video is sent automatically
    // user will click "send" button explicitly for text only
    await room.typingNotice(false);
    var eventId = await room.sendPlainMessage(message.text);
    final textMessage = types.TextMessage(
      author: user,
      createdAt: DateTime.now().millisecondsSinceEpoch,
      id: eventId,
      text: message.text,
      status: types.Status.sent,
      showStatus: true,
    );
    messages.insert(0, textMessage);
    update(['Chat']);
  }

  //image selection
  Future<void> handleImageSelection(BuildContext context) async {
    final result = await ImagePicker().pickImage(
      imageQuality: 70,
      maxWidth: 1440,
      source: ImageSource.gallery,
    );

    if (result != null) {
      final bytes = await result.readAsBytes();
      final image = await decodeImageFromList(bytes);
      final mimeType = lookupMimeType(result.path);
      var eventId = await room.sendImageMessage(
        result.path,
        result.name,
        mimeType!,
        bytes.length,
        image.width,
        image.height,
      );

      // i am sending message
      final message = types.ImageMessage(
        author: user,
        createdAt: DateTime.now().millisecondsSinceEpoch,
        height: image.height.toDouble(),
        id: eventId,
        name: result.name,
        size: bytes.length,
        uri: result.path,
        width: image.width.toDouble(),
      );
      Navigator.pop(context);
      messages.insert(0, message);
      update(['Chat']);
    }
  }

  //file selection
  Future<void> handleFileSelection(BuildContext context) async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.any,
    );

    if (result != null && result.files.single.path != null) {
      final mimeType = lookupMimeType(result.files.single.path!);
      await room.sendFileMessage(
        result.files.single.path!,
        result.files.single.name,
        mimeType!,
        result.files.single.size,
      );

      // i am sending message
      final message = types.FileMessage(
        author: user,
        createdAt: DateTime.now().millisecondsSinceEpoch,
        id: randomString(),
        name: result.files.single.name,
        size: result.files.single.size,
        uri: result.files.single.path!,
      );
      Navigator.pop(context);
      messages.insert(0, message);
      update(['Chat']);
    }
  }

  Future<void> handleMessageTap(
    BuildContext context,
    types.Message message,
  ) async {
    if (message is types.FileMessage) {
      String filePath = await room.filePath(message.id);
      if (filePath.isEmpty) {
        Directory? rootPath = await getTemporaryDirectory();
        String? dirPath = await FilesystemPicker.open(
          title: 'Save to folder',
          context: context,
          rootDirectory: rootPath,
          fsType: FilesystemType.folder,
          pickText: 'Save file to this folder',
          folderIconColor: Colors.teal,
          requestPermission: !_isDesktop
              ? () async => await Permission.storage.request().isGranted
              : null,
        );
        if (dirPath != null) {
          await room.saveFile(message.id, dirPath);
        }
      } else {
        final result = await OpenFile.open(filePath);
        if (result.message.isNotEmpty) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(result.message),
            ),
          );
        }
      }
    }
  }

  //Pagination Control
  Future<void> handleEndReached() async {
    final msgs = await _stream!.paginateBackwards(10);
    final List<types.Message> nextMessages = [];
    // i am fetching messages from remote
    for (RoomMessage message in msgs) {
      _loadMessage(message, nextMessages);
    }
    messages = [...messages, ...nextMessages];
    _page = _page + 1;
    update(['Chat']);
  }

  void _insertMessage(types.Message m, List<types.Message> c) {
    for (var i = 0; i < c.length; i++) {
      if (c[i].createdAt! < m.createdAt!) {
        c.insert(i, m);
        return;
      }
    }
    c.add(m);
  }

  void _loadMessage(RoomMessage message, List<types.Message> container) {
    String msgtype = message.msgtype();
    if (msgtype == 'm.audio') {
    } else if (msgtype == 'm.emote') {
    } else if (msgtype == 'm.file') {
      FileDescription? description = message.fileDescription();
      if (description != null) {
        types.FileMessage m = types.FileMessage(
          author: types.User(id: message.sender()),
          createdAt: message.originServerTs() * 1000,
          id: message.eventId(),
          name: description.name(),
          size: description.size() ?? 0,
          uri: '',
        );
        _insertMessage(m, container);
      }
      if (isLoading.isFalse) {
        update(['Chat']);
      }
    } else if (msgtype == 'm.image') {
      String eventId = message.eventId();
      ImageDescription? description = message.imageDescription();
      if (description != null) {
        types.ImageMessage m = types.ImageMessage(
          author: types.User(id: message.sender()),
          createdAt: message.originServerTs() * 1000,
          height: description.height()?.toDouble(),
          id: eventId,
          name: description.name(),
          size: description.size() ?? 0,
          uri: '',
          width: description.width()?.toDouble(),
        );
        _insertMessage(m, container);
        if (isLoading.isFalse) {
          update(['Chat']);
        }
        room.imageBinary(eventId).then((data) async {
          await mtx.acquire();
          for (var i = 0; i < messages.length; i++) {
            if (messages[i].id == eventId) {
              messages[i] = messages[i].copyWith(
                metadata: {
                  'binary': data.asTypedList(),
                },
              );
              break;
            }
          }
          update(['Chat']);
          mtx.release();
        });
      }
    } else if (msgtype == 'm.location') {
    } else if (msgtype == 'm.notice') {
    } else if (msgtype == 'm.server_notice') {
    } else if (msgtype == 'm.text') {
      String sender = message.sender();
      types.TextMessage m = types.TextMessage(
        author: sender == user.id ? user : types.User(id: sender),
        createdAt: message.originServerTs() * 1000,
        id: message.eventId(),
        text: message.body(),
      );
      _insertMessage(m, container);
      if (isLoading.isFalse) {
        update(['Chat']);
      }
    } else if (msgtype == 'm.video') {
    } else if (msgtype == 'm.key.verification.request') {}
  }
}
