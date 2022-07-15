// ignore_for_file: prefer_const_constructors, avoid_unnecessary_containers, sized_box_for_whitespace, prefer_final_fields, prefer_typing_uninitialized_variables
import 'package:effektio/common/store/separatedThemes.dart';
import 'package:effektio/common/widget/ChatOverview.dart';
import 'package:effektio/common/widget/InviteInfoWidget.dart';
import 'package:effektio_flutter_sdk/effektio_flutter_sdk_ffi.dart'
    show Client, FfiListConversation, FfiListInvitation, Invitation;
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:themed/themed.dart';

class ChatList extends StatefulWidget {
  const ChatList({Key? key, required this.client}) : super(key: key);
  final Future<Client> client;

  @override
  State<ChatList> createState() => _ChatListState();
}

class _ChatListState extends State<ChatList> {
  String? user;

  @override
  void initState() {
    super.initState();
    _getUser().whenComplete(() => setState(() {}));
  }

  @override
  void dispose() {
    super.dispose();
  }

  Future<void> _getUser() async {
    var fetchClient = await widget.client;
    user = await fetchClient.userId().then((u) => u.toString());
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leadingWidth: MediaQuery.of(context).size.width * 0.2,
        toolbarHeight: MediaQuery.of(context).size.height / 15,
        elevation: 0,
        leading: FittedBox(
          fit: BoxFit.scaleDown,
          child: TextButton(
            onPressed: () {},
            child: Text(
              AppLocalizations.of(context)!.select,
              style: ChatTheme01.chatTitleStyle + AppCommonTheme.primaryColor,
              textAlign: TextAlign.center,
            ),
          ),
        ),
        actions: [
          IconButton(
            onPressed: () {},
            padding: const EdgeInsets.only(
              right: 10,
            ),
            icon: SvgPicture.asset(
              'assets/images/edit.svg',
              color: AppCommonTheme.svgIconColor,
              width: 20,
              height: 20,
            ),
          ),
        ],
      ),
      body: Container(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Container(
              margin: const EdgeInsets.only(
                left: 18,
              ),
              child: Text(
                AppLocalizations.of(context)!.chat,
                style: AppCommonTheme.appBartitleStyle,
              ),
            ),
            SizedBox(height: 10),
            Expanded(
              child: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    buildInvitedRooms(),
                    buildJoinedRooms(),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget buildInvitedRooms() {
    return FutureBuilder<Client>(
      future: widget.client,
      builder: (BuildContext context, AsyncSnapshot<Client> snapshot) {
        if (!snapshot.hasData) {
          return SizedBox.shrink();
        }
        FfiListInvitation pendingInvitations =
            snapshot.requireData.pendingInvitations();
        if (pendingInvitations.isEmpty) {
          return SizedBox.shrink();
        }
        // user may be invited from multiple people of same room
        var seen = <String>{};
        List<Invitation> distinct = pendingInvitations
            .where((item) => seen.add(item.getRoomId()))
            .toList();
        return Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              alignment: Alignment.topLeft,
              padding: const EdgeInsets.only(left: 18),
              child: Text(
                AppLocalizations.of(context)!.invitedRooms,
                style: AppCommonTheme.appBartitleStyle.copyWith(fontSize: 16),
              ),
            ),
            SizedBox(height: 10),
            ListView.builder(
              physics: NeverScrollableScrollPhysics(),
              shrinkWrap: true,
              itemCount: distinct.length,
              itemBuilder: (BuildContext context, int index) {
                return Container(
                  height: MediaQuery.of(context).size.height / 5,
                  width: MediaQuery.of(context).size.width,
                  child: InviteInfoWidget(
                    avatarColor: Colors.white,
                    inviter: distinct[index].getSender() ?? 'Unknown',
                    groupName: distinct[index].getRoomName(),
                  ),
                );
              },
            ),
          ],
        );
      },
    );
  }

  Widget buildJoinedRooms() {
    return FutureBuilder<Client>(
      future: widget.client,
      builder: (BuildContext context, AsyncSnapshot<Client> snapshot) {
        if (snapshot.connectionState != ConnectionState.done) {
          return SizedBox(
            height: MediaQuery.of(context).size.height / 1.5,
            width: MediaQuery.of(context).size.width,
            child: Center(
              child: CircularProgressIndicator(
                color: AppCommonTheme.primaryColor,
              ),
            ),
          );
        }
        if (snapshot.hasData) {
          return FutureBuilder<FfiListConversation>(
            future: snapshot.requireData.conversations(),
            builder: (
              BuildContext context,
              AsyncSnapshot<FfiListConversation> snapshot,
            ) {
              if (snapshot.hasData) {
                return Flexible(
                  child: ChatOverview(
                    user: user,
                    rooms: snapshot.requireData.toList(),
                  ),
                );
              } else {
                return Center(
                  child: Container(
                    height: MediaQuery.of(context).size.height,
                    width: MediaQuery.of(context).size.width,
                    color: AppCommonTheme.backgroundColor,
                    child: Text(
                      AppLocalizations.of(context)!.loadingConvo,
                      style: ChatTheme01.emptyMsgTitle,
                    ),
                  ),
                );
              }
            },
          );
        } else {
          return Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: <Widget>[
              SizedBox(height: MediaQuery.of(context).size.height / 6),
              Center(
                child: Container(
                  child: SvgPicture.asset('assets/images/empty_messages.svg'),
                ),
              ),
              SizedBox(height: 20),
              Text(
                AppLocalizations.of(context)!.loadingConvo + '...',
                style: ChatTheme01.emptyMsgTitle,
              ),
              SizedBox(height: 5),
              Center(
                child: Container(
                  height: MediaQuery.of(context).size.height / 3,
                  width: MediaQuery.of(context).size.width / 1.5,
                  child: Text(
                    'Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.',
                    style: ChatTheme01.chatBodyStyle,
                    overflow: TextOverflow.clip,
                    textAlign: TextAlign.center,
                  ),
                ),
              ),
            ],
          );
        }
      },
    );
  }
}
