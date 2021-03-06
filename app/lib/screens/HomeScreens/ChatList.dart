// ignore_for_file: prefer_const_constructors, avoid_unnecessary_containers, sized_box_for_whitespace, prefer_final_fields, prefer_typing_uninitialized_variables

import 'dart:math';

import 'package:effektio/common/store/separatedThemes.dart';
import 'package:effektio/common/widget/ChatOverview.dart';
import 'package:effektio/common/widget/InviteInfoWidget.dart';
import 'package:effektio_flutter_sdk/effektio_flutter_sdk_ffi.dart'
    show Client, FfiListConversation;
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:themed/themed.dart';

import 'package:effektio/common/store/MockData.dart';

class ChatList extends StatefulWidget {
  const ChatList({Key? key, required this.client}) : super(key: key);
  final Future<Client> client;

  @override
  State<ChatList> createState() => _ChatListState();
}

class _ChatListState extends State<ChatList> {
  String? user;
  late final countInvites;
  Random random = Random();
  @override
  void initState() {
    //setting random invites
    countInvites = random.nextInt(5) + 1;
    super.initState();
    _getUser().whenComplete(() => {setState(() {})});
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
      body: CustomScrollView(
        physics: BouncingScrollPhysics(),
        slivers: [
          SliverAppBar(
            leading: TextButton(
              onPressed: () {},
              child: Container(
                margin: const EdgeInsets.only(right: 15),
                child: Text(
                  AppLocalizations.of(context)!.select,
                  style:
                      ChatTheme01.chatTitleStyle + AppCommonTheme.primaryColor,
                ),
              ),
            ),
            leadingWidth: 100,
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
          SliverToBoxAdapter(
            child: Container(
              child: Column(
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
                  Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Container(
                        alignment: Alignment.topLeft,
                        padding: const EdgeInsets.only(left: 18),
                        child: Text(
                          AppLocalizations.of(context)!.invites,
                          style: AppCommonTheme.appBartitleStyle
                              .copyWith(fontSize: 16),
                        ),
                      ),
                      SizedBox(height: 10),
                      ListView.builder(
                        physics: NeverScrollableScrollPhysics(),
                        shrinkWrap: true,
                        itemCount: countInvites,
                        itemBuilder: (BuildContext context, int index) {
                          return Container(
                            child: InviteInfoWidget(
                              avatarColor: Colors.white,
                              inviter:
                                  inviters[random.nextInt(inviters.length)],
                              groupName: groups[random.nextInt(groups.length)],
                            ),
                          );
                        },
                      ),
                      FutureBuilder<Client>(
                        future: widget.client,
                        builder: (
                          BuildContext context,
                          AsyncSnapshot<Client> snapshot,
                        ) {
                          if (snapshot.connectionState !=
                              ConnectionState.done) {
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
                              future: snapshot.requireData
                                  .conversations(), // a previously-obtained Future<String> or null
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
                                      height:
                                          MediaQuery.of(context).size.height,
                                      width: MediaQuery.of(context).size.width,
                                      color: AppCommonTheme.backgroundColor,
                                      child: Text(
                                        AppLocalizations.of(context)!
                                            .loadingConvo,
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
                                SizedBox(
                                  height:
                                      MediaQuery.of(context).size.height / 6,
                                ),
                                Center(
                                  child: Container(
                                    child: SvgPicture.asset(
                                      'assets/images/empty_messages.svg',
                                    ),
                                  ),
                                ),
                                SizedBox(
                                  height: 20,
                                ),
                                Text(
                                  AppLocalizations.of(context)!.loadingConvo +
                                      '...',
                                  style: ChatTheme01.emptyMsgTitle,
                                ),
                                SizedBox(
                                  height: 5,
                                ),
                                Center(
                                  child: Container(
                                    height:
                                        MediaQuery.of(context).size.height / 3,
                                    width:
                                        MediaQuery.of(context).size.width / 1.5,
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
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
