object Invitation {
    fn get_event_id() -> string;
    fn get_sender() -> string;
}

/// Initialize logging
fn init_logging(filter: Option<string>) -> Result<()>;

/// Create a new client for homeserver at url with storage at data_path
fn login_new_client(basepath: string, username: string, password: string) -> Future<Result<Client>>;

/// Create a new client from the restore token
fn login_with_token(basepath: string, restore_token: string) -> Future<Result<Client>>;

/// Create a new client anonymous client connecting to the homeserver
fn guest_client(basepath: string, homeserver: string) -> Future<Result<Client>>;

/// Create a new client from the restore token
fn register_with_registration_token(basepath: string, username: string, password: string, registration_token: string) -> Future<Result<Client>>;

/// generate news mock items
fn gen_mock_news() -> Vec<News>;


/// Representing a color
object Color {
    /// as rgba in u8
    fn rgba_u8() -> (u8, u8, u8, u8);
}

/// A news object
object News {
    /// get the text of the news item
    fn text() -> Option<string>;
    /// the tags on this item
    fn tags() -> Vec<Tag>;
    /// the number of likes on this item
    fn likes_count() -> u32;
    /// the number of comments on this item
    fn comments_count() -> u32;
    /// if given, the specific foreground color
    fn fg_color() -> Option<Color>; 
    /// if given, the specific background color
    fn bg_color() -> Option<Color>; 
    /// if given, the image
    fn image() -> Option<Vec<u8>>; 
}

object Tag {
    /// the title of the tag
    fn title() -> string;
    /// dash-cased-ascii-version for usage in hashtags (no `#` at the front)
    fn hash_tag() -> string;
    /// if given, the specific color for this tag
    fn color() -> Option<Color>; 
}

/// A news object
object Faq {
    /// get the title of the news item
    fn title() -> string;
    /// get the body of the news item
    fn body() -> string;
    /// whether this object is pinned
    fn pinned() -> bool;
    // The team this faq belongs to
    //fn team() -> Room;
    /// the tags on this item
    fn tags() -> Vec<Tag>;
    /// the number of likes on this item
    fn likes_count() -> u32;
    /// the number of comments on this item
    fn comments_count() -> u32;
}

object UserId {
    // full name as string
    //fn as_str() -> string;

    fn to_string() -> string;

    // only the user name itself
    //fn localpart() -> string;
}

object EventId {}

/// A room Message metadata and content
object RoomMessage {

    /// Unique ID of this event
    fn event_id() -> string;

    /// The User, who sent that event
    fn sender() -> string;

    /// the body of the massage - fallback string reprensentation
    fn body() -> string;

    /// the server receiving timestamp
    fn origin_server_ts() -> u64;

    /// the type of massage, like audio, text, image, file, etc
    fn msgtype() -> string;

    /// contains source data, name, mimetype, size, width and height
    fn image_description() -> Option<ImageDescription>;

    /// contains source data, name, mimetype and size
    fn file_description() -> Option<FileDescription>;
}

object ImageDescription {

    /// file name
    fn name() -> string;

    /// MIME
    fn mimetype() -> Option<string>;

    /// file size in bytes
    fn size() -> Option<u64>;

    /// image width
    fn width() -> Option<u64>;

    /// image height
    fn height() -> Option<u64>;
}

object FileDescription {

    /// file name
    fn name() -> string;

    /// MIME
    fn mimetype() -> Option<string>;

    /// file size in bytes
    fn size() -> Option<u64>;
}

/// Timeline with Room Events
object TimelineStream {
    /// Fires whenever a new event arrived
    fn next() -> Future<Result<RoomMessage>>;

    /// Get the next count messages backwards,
    fn paginate_backwards(count: u64) -> Future<Result<Vec<RoomMessage>>>;
}

/// History with Invitation Events
object InvitationStream {
    /// Fires whenever a new event arrived
    fn next() -> Future<Result<Invitation>>;

    /// Get the next count events backwards,
    fn paginate_backwards(count: u64) -> Future<Result<Vec<Invitation>>>;
}

object Conversation {
    /// Calculate the display name
    fn display_name() -> Future<Result<string>>;

    /// The avatar of the room
    fn avatar() -> Future<Result<buffer<u8>>>;

    /// the members currently in the room
    fn active_members() -> Future<Result<Vec<Member>>>;

    /// Get the timeline for the room
    fn timeline() -> Future<Result<TimelineStream>>;

    /// Get the invitation history for the room
    fn invitation_history() -> Future<Result<InvitationStream>>;

    // the members currently in the room
    fn get_member(user_id: UserId) -> Future<Result<Member>>;

    /// The last message sent to the room
    fn latest_message() -> Future<Result<RoomMessage>>;

    /// Activate typing notice for this room
    /// The typing notice remains active for 4s. It can be deactivate at any
    /// point by setting typing to false. If this method is called while the
    /// typing notice is active nothing will happen. This method can be called
    /// on every key stroke, since it will do nothing while typing is active.
    fn typing_notice(typing: bool) -> Future<Result<bool>>;

    /// Send a request to notify this room that the user has read specific event.
    fn read_receipt(event_id: string) -> Future<Result<bool>>;

    /// Send a simple plain text message to the room
    /// returns the event_id as given by the server of the event soon after
    /// received over timeline().next()
    fn send_plain_message(text_message: string) -> Future<Result<string>>;

    fn send_image_message(uri: string, name: string, mimetype: string, size: Option<u32>, width: Option<u32>, height: Option<u32>) -> Future<Result<string>>;

    /// decrypted image file data
    /// The reason that this function belongs to room object is because ChatScreen keeps it as member variable
    /// If this function belongs to message object, we may have to load too many message objects in ChatScreen
    fn image_binary(event_id: string) -> Future<Result<buffer<u8>>>;

    fn send_file_message(uri: string, name: string, mimetype: string, size: u32) -> Future<Result<string>>;

    /// save file in specified path
    fn save_file(event_id: string, dir_path: string) -> Future<Result<string>>;

    /// get the path that file was saved
    fn file_path(event_id: string) -> Future<Result<string>>;
}

object Group {
    /// Calculate the display name
    fn display_name() -> Future<Result<string>>;

    /// The avatar of the Group
    fn avatar() -> Future<Result<buffer<u8>>>;

    /// the members currently in the group
    fn active_members() -> Future<Result<Vec<Member>>>;

    // the members currently in the room
    fn get_member(user: UserId) -> Future<Result<Member>>;
}

object Member {

    /// The avatar of the member
    fn avatar() -> Future<Result<buffer<u8>>>;

    /// Calculate the display name
    fn display_name() -> Option<string>;

    /// Full user_id
    fn user_id() -> UserId;

}

object Account {
    /// The display_name of the account
    fn display_name() -> Future<Result<string>>;

    /// Change the display name of the account
    fn set_display_name(name: string) -> Future<Result<bool>>;

    /// The avatar of the client
    fn avatar() -> Future<Result<buffer<u8>>>;

    /// Change the avatar of the account
    /// provide the c_type as MIME, e.g. `image/jpeg`
    fn set_avatar(c_type: string, data: Vec<u8>) -> Future<Result<bool>>;
}



/// Main entry point for `effektio`.
object Client {
    // Special

    /// Get the restore token for this session
    fn restore_token() -> Future<Result<string>>;

    /// Whether the client is registered as a guest account
    fn is_guest() -> bool;

    /// Whether the client has finished a first sync run
    fn has_first_synced() -> bool;

    /// Whether the client is syncing
    fn is_syncing() -> bool;

    /// Whether the client is logged in
    fn logged_in() -> Future<bool>;

    /// return the account of the logged in user, if given
    fn account() -> Future<Result<Account>>;

    // The device_id of the client
    fn device_id() -> Future<Result<string>>;

    /// The user_id of the client
    /// deprecated, please use account() instead.
    fn user_id() -> Future<Result<UserId>>;

    /// The display_name of the client
    /// deprecated, please use account() instead.
    fn display_name() -> Future<Result<string>>;

    /// The avatar of the client
    /// deprecated, please use account() instead.
    fn avatar() -> Future<Result<buffer<u8>>>;

    /// The conversations the user is involved in
    fn conversations() -> Future<Result<Vec<Conversation>>>;

    /// The groups the user is part of
    fn groups() -> Future<Result<Vec<Group>>>;

    /// Get the latest News for the client
    fn latest_news() -> Future<Result<Vec<News>>>;

    /// Get the FAQs for the client
    fn faqs() -> Future<Result<Vec<Faq>>>;
}
