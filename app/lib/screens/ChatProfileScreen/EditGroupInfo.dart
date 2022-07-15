import 'package:effektio/common/store/separatedThemes.dart';
import 'package:effektio/common/widget/customAvatar.dart';
import 'package:effektio/controllers/chat_controller.dart';
import 'package:effektio_flutter_sdk/effektio_flutter_sdk_ffi.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

class EditGroupInfoScreen extends StatefulWidget {
  final Conversation room;
  final String name;
  final String description;

  const EditGroupInfoScreen({
    Key? key,
    required this.room,
    required this.name,
    required this.description,
  }) : super(key: key);

  @override
  _EditGroupInfoState createState() => _EditGroupInfoState();
}

class _EditGroupInfoState extends State<EditGroupInfoScreen> {
  TextEditingController nameController = TextEditingController();
  TextEditingController descController = TextEditingController();
  ChatController chatController = Get.find<ChatController>();

  @override
  void initState() {
    super.initState();
    nameController.text = widget.name;
    descController.text = widget.description;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        elevation: 0.0,
        backgroundColor: AppCommonTheme.backgroundColor,
      ),
      body: Padding(
        padding: const EdgeInsets.all(12.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            GestureDetector(
              onTap: () {
                chatController.handleImageSelection(context);
              },
              child: Center(
                child: Padding(
                  padding: const EdgeInsets.only(top: 38.0, bottom: 12.0),
                  child: SizedBox(
                    height: 100,
                    width: 100,
                    child: FittedBox(
                      fit: BoxFit.contain,
                      child: CustomAvatar(
                        avatar: widget.room.avatar(),
                        displayName: widget.room.displayName(),
                        radius: 20,
                        isGroup: true,
                        stringName: '',
                      ),
                    ),
                  ),
                ),
              ),
            ),
            Container(
              margin: const EdgeInsets.only(top: 20.0, bottom: 10.0),
              child: const Text(
                'Group Name',
                style: TextStyle(color: Colors.white, fontSize: 16.0),
              ),
            ),
            SingleChildScrollView(
              child: TextField(
                style: const TextStyle(color: Colors.white),
                controller: nameController,
                decoration: const InputDecoration(
                  isCollapsed: true,
                  contentPadding: EdgeInsets.all(12),
                  filled: true,
                  fillColor: AppCommonTheme.darkShade,
                ),
              ),
            ),
            Container(
              margin: const EdgeInsets.only(top: 20.0, bottom: 10.0),
              child: const Text(
                'Group Description',
                style: TextStyle(color: Colors.white, fontSize: 16.0),
              ),
            ),
            Flexible(
              child: TextField(
                keyboardType: TextInputType.multiline,
                maxLines: null,
                style: const TextStyle(color: Colors.white),
                controller: descController,
                decoration: const InputDecoration(
                  contentPadding: EdgeInsets.all(12),
                  isCollapsed: false,
                  filled: true,
                  fillColor: AppCommonTheme.darkShade,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
