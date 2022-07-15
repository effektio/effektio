// ignore_for_file: prefer_const_constructors, avoid_unnecessary_containers

import 'package:effektio/common/store/separatedThemes.dart';
import 'package:effektio/common/widget/AppCommon.dart';
import 'package:effektio/common/widget/customAvatar.dart';
import 'package:effektio/screens/HomeScreens/ChatScreen.dart';
import 'package:effektio_flutter_sdk/effektio_flutter_sdk_ffi.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:intl/intl.dart';

class ChatOverview extends StatelessWidget {
  final List<Conversation> rooms;
  final String? user;

  const ChatOverview({
    Key? key,
    required this.rooms,
    required this.user,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    Iterable<Conversation> invitedRooms =
        rooms.where((element) => element.roomType() == 'invited');
    if (invitedRooms.isEmpty) {
      // hide title
      return buildListView(context);
    }
    // show title
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          alignment: Alignment.topLeft,
          padding: const EdgeInsets.only(left: 18),
          child: Text(
            AppLocalizations.of(context)!.joinedRooms,
            style: AppCommonTheme.appBartitleStyle.copyWith(fontSize: 16),
          ),
        ),
        SizedBox(height: 10),
        buildListView(context),
      ],
    );
  }

  Widget buildListView(BuildContext context) {
    List<Conversation> joinedRooms =
        rooms.where((element) => element.roomType() == 'joined').toList();
    return ListView.builder(
      physics: NeverScrollableScrollPhysics(),
      shrinkWrap: true,
      padding: const EdgeInsets.only(top: 10),
      itemCount: joinedRooms.length,
      itemBuilder: (BuildContext context, int index) {
        return Column(
          children: <Widget>[
            ChatListItem(
              room: joinedRooms[index],
              user: user,
            ),
            Divider(
              height: 1,
            ),
          ],
        );
      },
    );
  }
}

class ChatListItem extends StatelessWidget {
  final Conversation room;
  final String? user;
  const ChatListItem({Key? key, required this.room, required this.user})
      : super(key: key);

  @override
  Widget build(BuildContext context) {
    // ToDo: UnreadCounter
    return ListTile(
      onTap: () {
        Navigator.push(
          context,
          MaterialPageRoute(
            builder: (context) => ChatScreen(
              room: room,
              user: user,
            ),
          ),
        );
      },
      leading: CustomAvatar(
        avatar: room.avatar(),
        displayName: room.displayName(),
        radius: 25,
        isGroup: true,
        stringName: '',
      ),
      title: FutureBuilder<String>(
        future: room.displayName(),
        builder: (BuildContext context, AsyncSnapshot<String> snapshot) {
          if (snapshot.hasData) {
            String title = snapshot.requireData;
            return Text(
              title,
              style: ChatTheme01.chatTitleStyle,
            );
          } else {
            return Text(AppLocalizations.of(context)!.loadingName);
          }
        },
      ),
      subtitle: FutureBuilder<RoomMessage>(
        future: room.latestMessage(),
        builder: (BuildContext context, snapshot) {
          if (snapshot.hasData) {
            return Container(
              margin: const EdgeInsets.only(top: 10, bottom: 10),
              child: Text(
                '${getNameFromId(snapshot.requireData.sender())}: ${snapshot.requireData.body()}',
                style: ChatTheme01.latestChatStyle,
                maxLines: 2,
              ),
            );
          } else {
            return Container();
          }
        },
      ),
      trailing: FutureBuilder<RoomMessage>(
        future: room.latestMessage(),
        builder: (context, snapshot) {
          if (snapshot.hasData) {
            return Text(
              DateFormat.Hm().format(
                DateTime.fromMillisecondsSinceEpoch(
                  snapshot.requireData.originServerTs() * 1000,
                  isUtc: true,
                ),
              ),
              style: ChatTheme01.latestChatDateStyle,
            );
          } else {
            return const SizedBox();
          }
        },
      ),
    );
  }
}
