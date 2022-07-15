// ignore_for_file: prefer_const_constructors, sized_box_for_whitespace, prefer_final_fields, prefer_typing_uninitialized_variables

import 'dart:io';
import 'dart:math';
import 'package:effektio/common/store/MockData.dart';
import 'package:effektio/common/store/chatTheme.dart';
import 'package:effektio/common/store/separatedThemes.dart';
import 'package:effektio/common/widget/AppCommon.dart';
import 'package:effektio/common/widget/InviteInfoWidget.dart';
import 'package:effektio/common/widget/customAvatar.dart';
import 'package:effektio/common/widget/emptyMessagesPlaceholder.dart';
import 'package:effektio/controllers/chat_controller.dart';
import 'package:effektio/screens/ChatProfileScreen/ChatProfile.dart';
import 'package:effektio_flutter_sdk/effektio_flutter_sdk_ffi.dart'
    show Conversation, FfiListMember;
import 'package:another_flushbar/flushbar.dart';
import 'package:flutter/material.dart';
import 'package:flutter_chat_types/flutter_chat_types.dart' as types;
import 'package:flutter_chat_ui/flutter_chat_ui.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:get/get.dart';
import 'package:string_validator/string_validator.dart';
import 'package:themed/themed.dart';
import 'package:transparent_image/transparent_image.dart';

class ChatScreen extends StatefulWidget {
  final Conversation room;
  final String? user;
  const ChatScreen({Key? key, required this.room, required this.user})
      : super(key: key);

  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  late final _user;
  String roomName = '';
  bool roomState = false;
  final Random random = Random();
  ChatController chatController = Get.find<ChatController>();

  @override
  void initState() {
    _user = types.User(
      id: widget.user!,
    );
    //roomState is true in case of invited and false if already joined
    //has some restrictions in case of true i.e.send option is disabled. You can set it permanantly false or true for testing
    roomState = random.nextBool();
    chatController.init(widget.room, _user);
    super.initState();
  }

  @override
  void dispose() {
    Get.delete<ChatController>();
    super.dispose();
  }

  void _handleAttachmentPressed(BuildContext context) {
    showModalBottomSheet<void>(
      backgroundColor: Colors.transparent,
      context: context,
      builder: (BuildContext context) {
        return SafeArea(
          child: SizedBox(
            height: 124,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                GestureDetector(
                  onTap: () => chatController.handleImageSelection(context),
                  child: Row(
                    children: <Widget>[
                      Padding(
                        padding: const EdgeInsets.all(8.0),
                        child: SvgPicture.asset('assets/images/camera.svg'),
                      ),
                      SizedBox(width: 10),
                      Padding(
                        padding: const EdgeInsets.all(8.0),
                        child: Text(
                          AppLocalizations.of(context)!.photo,
                          style: TextStyle(color: Colors.white),
                        ),
                      )
                    ],
                  ),
                ),
                SizedBox(height: 10),
                GestureDetector(
                  onTap: () => chatController.handleFileSelection(context),
                  child: Row(
                    children: <Widget>[
                      Padding(
                        padding: const EdgeInsets.all(8.0),
                        child: SvgPicture.asset('assets/images/document.svg'),
                      ),
                      SizedBox(width: 10),
                      Padding(
                        padding: const EdgeInsets.all(8.0),
                        child: Text(
                          AppLocalizations.of(context)!.file,
                          style: TextStyle(
                            color: Colors.white,
                          ),
                        ),
                      )
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _avatarBuilder(String userId) {
    return GetBuilder<ChatController>(
      id: 'Avatar',
      builder: (ChatController controller) {
        return Padding(
          padding: const EdgeInsets.only(right: 10),
          child: CustomAvatar(
            avatar: widget.room.avatar(),
            displayName: null,
            radius: 15,
            isGroup: false,
            stringName: getNameFromId(userId),
          ),
        );
      },
    );
  }

  Widget _imageMessageBuilder(
    types.ImageMessage imageMessage, {
    required int messageWidth,
  }) {
    if (imageMessage.uri.isEmpty) {
      // binary data
      if (imageMessage.metadata?.containsKey('binary') ?? false) {
        return Image.memory(
          imageMessage.metadata?['binary'],
          width: messageWidth.toDouble(),
        );
      } else {
        return Image.memory(
          kTransparentImage,
          width: messageWidth.toDouble(),
        );
      }
    } else if (isURL(imageMessage.uri)) {
      // remote url
      return Image.network(
        imageMessage.uri,
        width: messageWidth.toDouble(),
      );
    } else {
      // local path
      // the image that just sent is displayed from local not remote
      return Image.file(
        File(imageMessage.uri),
        width: messageWidth.toDouble(),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: AppCommonTheme.backgroundColor,
        elevation: 1,
        centerTitle: true,
        toolbarHeight: 70,
        leading: Row(
          children: <Widget>[
            IconButton(
              onPressed: () {
                Navigator.pop(context);
              },
              icon: SvgPicture.asset(
                'assets/images/back_button.svg',
                color: AppCommonTheme.svgIconColor,
              ),
            ),
          ],
        ),
        title: Column(
          mainAxisSize: MainAxisSize.max,
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            FutureBuilder<String>(
              future:
                  widget.room.displayName().then((value) => roomName = value),
              builder: (BuildContext context, AsyncSnapshot<String> snapshot) {
                if (snapshot.hasData) {
                  return Text(
                    snapshot.requireData,
                    overflow: TextOverflow.clip,
                    style: ChatTheme01.chatTitleStyle,
                  );
                } else {
                  return Text(AppLocalizations.of(context)!.loadingName);
                }
              },
            ),
            SizedBox(height: 5),
            FutureBuilder<FfiListMember>(
              future: widget.room.activeMembers(),
              builder: (
                BuildContext context,
                AsyncSnapshot<FfiListMember> snapshot,
              ) {
                if (snapshot.hasData) {
                  return Text(
                    '${snapshot.requireData.length.toString()} ${AppLocalizations.of(context)!.members}',
                    style:
                        ChatTheme01.chatBodyStyle + AppCommonTheme.primaryColor,
                  );
                } else {
                  return Container(
                    height: 15,
                    width: 15,
                    child: CircularProgressIndicator(
                      color: AppCommonTheme.primaryColor,
                    ),
                  );
                }
              },
            ),
          ],
        ),
        actions: [
          GestureDetector(
            onTap: () {
              Navigator.push(
                context,
                MaterialPageRoute(
                  builder: (context) => ChatProfileScreen(
                    room: widget.room,
                    user: widget.user,
                    isGroup: true,
                    isAdmin: true,
                  ),
                ),
              );
            },
            child: Padding(
              padding: const EdgeInsets.only(right: 10),
              child: Container(
                height: 45,
                width: 45,
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
        ],
      ),
      body: Obx(
        () => SafeArea(
          bottom: false,
          child: _buildBody(context),
        ),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    if (chatController.isLoading.isTrue) {
      return Center(
        child: Container(
          height: 15,
          width: 15,
          child: CircularProgressIndicator(
            color: AppCommonTheme.primaryColor,
          ),
        ),
      );
    }
    return GetBuilder<ChatController>(
      id: 'Chat',
      init: chatController,
      builder: (ChatController controller) {
        return Stack(
          children: [
            Chat(
              l10n: ChatL10nEn(
                emptyChatPlaceholder: '',
                attachmentButtonAccessibilityLabel: '',
                fileButtonAccessibilityLabel: '',
                inputPlaceholder: AppLocalizations.of(context)!.message,
                sendButtonAccessibilityLabel: '',
              ),
              messages: chatController.messages,
              sendButtonVisibilityMode: roomState
                  ? SendButtonVisibilityMode.hidden
                  : SendButtonVisibilityMode.editing,
              onSendPressed: chatController.handleSendPressed,
              user: _user,
              disableImageGallery: roomState ? true : false,
              //custom avatar builder
              avatarBuilder: _avatarBuilder,
              imageMessageBuilder: _imageMessageBuilder,
              //Whenever users starts typing on keyboard, this will trigger the function
              onTextChanged: (text) async {
                await controller.room.typingNotice(true);
              },
              showUserAvatars: true,
              onAttachmentPressed: () => _handleAttachmentPressed(context),
              onPreviewDataFetched: controller.handlePreviewDataFetched,
              onMessageTap: controller.handleMessageTap,
              onEndReached: roomState ? null : controller.handleEndReached,
              onEndReachedThreshold: 0.75,
              emptyState: EmptyPlaceholder(),
              //Custom Theme class, see lib/common/store/chatTheme.dart
              theme: EffektioChatTheme(
                attachmentButtonIcon:
                    SvgPicture.asset('assets/images/attachment.svg'),
                sendButtonIcon: SvgPicture.asset('assets/images/sendIcon.svg'),
                seenIcon: SvgPicture.asset('assets/images/seenIcon.svg'),
                deliveredIcon: SvgPicture.asset('assets/images/sentIcon.svg'),
              ),
            ),
            roomState
                ? Container(
                    alignment: Alignment.topLeft,
                    padding:
                        const EdgeInsets.only(top: 10, bottom: 20, left: 10),
                    color: AppCommonTheme.backgroundColor,
                    height: MediaQuery.of(context).size.height * 0.25,
                    width: MediaQuery.of(context).size.width,
                    child: Text(
                      AppLocalizations.of(context)!.invitationText1,
                      style: AppCommonTheme.appBartitleStyle
                          .copyWith(fontSize: 14),
                    ),
                  )
                : Container(),
            roomState
                ? Padding(
                    padding: const EdgeInsets.only(top: 40),
                    child: InviteInfoWidget(
                      avatarColor: Colors.white,
                      inviter: inviters[random.nextInt(inviters.length)],
                      groupName: roomName,
                    ),
                  )
                : Container(),
          ],
        );
      },
    );
  }
}
