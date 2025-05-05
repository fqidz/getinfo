// https://www.matthew.ath.cx/misc/dbus

#include <time.h>
#include <stdio.h>
#include <dbus/dbus.h>

// #define MPRIS_PATH "/org/mpris/MediaPlayer2"
#define PLAYER_IFACE "org.mpris.MediaPlayer2.Player"
// #define PROPERTIES_IFACE "org.freedesktop.DBus.Properties"

void property_get_reply(DBusPendingCall *call, void *user_data) {
    (void)user_data;
    DBusMessage *reply;
    DBusMessageIter iter;
    reply = dbus_pending_call_steal_reply(call);
    dbus_message_iter_init(reply, &iter);
    printf("balls\n");
}

int main(void)
{
    DBusConnection *connection = NULL;
    DBusError errors;
    dbus_error_init(&errors);

    connection = dbus_bus_get(DBUS_BUS_SESSION, &errors);

    dbus_uint32_t serial = 17;
    DBusMessage *message = NULL;
    DBusMessageIter args;

    // message = dbus_message_new_signal(MPRIS_PATH, const char *iface, const char *name)
    message = dbus_message_new_method_call(DBUS_INTERFACE_DBUS,
                                       DBUS_PATH_DBUS, DBUS_INTERFACE_DBUS,
                                       "Properties.Get");

    dbus_message_iter_init(message, &args);
    dbus_message_iter_append_basic(&args, DBUS_TYPE_STRING, PLAYER_IFACE);
    dbus_message_iter_append_basic(&args, DBUS_TYPE_STRING, "PlaybackStatus");

    DBusPendingCall *pending_return = NULL;
    dbus_connection_send(connection, message, &serial);
    dbus_connection_send_with_reply(connection, message, &pending_return, -1);

    void *data = NULL;

    dbus_pending_call_set_notify(pending_return,
                                 property_get_reply,
                                 data,
                                 dbus_free);
    struct timespec ts;
    ts.tv_sec = 0;
    ts.tv_nsec = 2e8;
    while (!dbus_pending_call_get_completed(pending_return)) {
        printf("%ui\n", dbus_pending_call_get_completed(pending_return));
        nanosleep(&ts, &ts);
        printf("sleeping\n");
    }

    dbus_connection_flush(connection);

    dbus_connection_close(connection);
    return 0;
}

