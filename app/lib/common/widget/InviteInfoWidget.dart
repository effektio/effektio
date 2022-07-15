// ignore_for_file: prefer_const_constructors, prefer_const_literals_to_create_immutables,

import 'package:flutter/material.dart';

import 'package:effektio/common/store/separatedThemes.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

class InviteInfoWidget extends StatelessWidget {
  const InviteInfoWidget({
    Key? key,
    required this.avatarColor,
    required this.inviter,
    required this.groupName,
  }) : super(key: key);
  final Color avatarColor;
  final String inviter;
  final String groupName;

  @override
  Widget build(BuildContext context) {
    return Card(
      color: AppCommonTheme.darkShade,
      margin: const EdgeInsets.only(top: 1, bottom: 1),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          ListTile(
            leading: CircleAvatar(backgroundColor: avatarColor),
            title: Text(
              inviter,
              style: AppCommonTheme.appBartitleStyle.copyWith(
                fontSize: 14,
                fontWeight: FontWeight.w500,
              ),
            ),
            subtitle: RichText(
              text: TextSpan(
                text: AppLocalizations.of(context)!.invitationText2,
                style: AppCommonTheme.appBartitleStyle.copyWith(
                  fontSize: 14,
                  fontWeight: FontWeight.w400,
                  color: AppCommonTheme.dividerColor,
                ),
                children: <TextSpan>[
                  TextSpan(
                    text: groupName,
                    style: AppCommonTheme.appBartitleStyle.copyWith(
                      fontSize: 14,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ],
              ),
            ),
          ),
          Divider(color: AppCommonTheme.dividerColor, indent: 15),
          Row(
            mainAxisAlignment: MainAxisAlignment.start,
            children: <Widget>[
              Container(
                width: MediaQuery.of(context).size.width * 0.48,
                padding: const EdgeInsets.only(left: 15),
                child: ElevatedButton(
                  onPressed: () {},
                  style: ElevatedButton.styleFrom(
                    primary: AppCommonTheme.greenButtonColor,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                  ),
                  child: Text(
                    AppLocalizations.of(context)!.accept,
                    style: AppCommonTheme.appBartitleStyle.copyWith(
                      fontSize: 14,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ),
              ),
              SizedBox(width: 15),
              Container(
                width: MediaQuery.of(context).size.width * 0.48,
                padding: const EdgeInsets.only(right: 15),
                child: ElevatedButton(
                  onPressed: () {},
                  style: ElevatedButton.styleFrom(
                    primary: AppCommonTheme.primaryColor,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                  ),
                  child: Text(
                    AppLocalizations.of(context)!.decline,
                    style: AppCommonTheme.appBartitleStyle.copyWith(
                      fontSize: 14,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
