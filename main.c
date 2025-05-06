// https://www.matthew.ath.cx/misc/dbus

#include "bits/time.h"
#include "time.h"
#include <math.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

#include <dbus/dbus.h>

typedef struct {
    char **items;
    unsigned int capacity;
    unsigned int length;
} StringArray;

void get_all_bus_names(DBusConnection *conn)
{
    DBusPendingCall *pending;
    {
        DBusMessage *msg =
                dbus_message_new_method_call(DBUS_SERVICE_DBUS, DBUS_PATH_DBUS,
                                             DBUS_SERVICE_DBUS, "ListNames");

        if (msg == NULL) {
            fprintf(stderr, "Message Null\n");
            exit(1);
        }

        // send message and get a handle for a reply
        if (!dbus_connection_send_with_reply(conn, msg, &pending,
                                             -1)) { // -1 is default timeout
            fprintf(stderr, "Out Of Memory!\n");
            exit(1);
        }
        if (NULL == pending) {
            fprintf(stderr, "Pending Call Null\n");
            exit(1);
        }
        dbus_connection_flush(conn);
        // free message
        dbus_message_unref(msg);
    }

    // block until we recieve a reply
    dbus_pending_call_block(pending);

    // get the reply message
    DBusMessage *reply = dbus_pending_call_steal_reply(pending);
    if (NULL == reply) {
        fprintf(stderr, "Reply Null\n");
        exit(1);
    }

    const char *reply_err_name = dbus_message_get_error_name(reply);
    if (reply_err_name) {
        printf("Error sending message: (%s)\n", reply_err_name);
        exit(1);
    }

    // free the pending message handle
    dbus_pending_call_unref(pending);

    // parse the reply
    DBusMessageIter reply_iter;
    DBusMessageIter reply_recurse;
    if (!dbus_message_iter_init(reply, &reply_iter)) {
        fprintf(stderr, "ERROR: Message has no arguments!\n");
        exit(1);
    }

    const char *current_name;
    dbus_message_iter_recurse(&reply_iter, &reply_recurse);
    do {
        dbus_message_iter_get_basic(&reply_recurse, &current_name);
#define MPRIS_BUS_PREFIX "org.mpris.MediaPlayer2."
        if (strncmp(MPRIS_BUS_PREFIX, current_name, strlen(MPRIS_BUS_PREFIX)) == 0) {
            printf("%s\n", current_name);
        }
    } while (dbus_message_iter_next(&reply_recurse));

    // free reply
    dbus_message_unref(reply);
}

void get_position(DBusConnection *conn)
{
    // create a new method call and check for errors
    DBusMessage *msg = dbus_message_new_method_call(
            "org.mpris.MediaPlayer2.spotify", // target for the method call
            "/org/mpris/MediaPlayer2", // object to call on
            "org.freedesktop.DBus.Properties", // interface to call on
            "Get"); // method name

    if (msg == NULL) {
        fprintf(stderr, "Message Null\n");
        exit(1);
    }

    DBusMessageIter args;

    char *interface_name = "org.mpris.MediaPlayer2.Player";
    char *property_name = "Position";
    // append arguments
    dbus_message_iter_init_append(msg, &args);
    if (!dbus_message_iter_append_basic(&args, DBUS_TYPE_STRING,
                                        &interface_name)) {
        fprintf(stderr, "Out Of Memory!\n");
        exit(1);
    }
    if (!dbus_message_iter_append_basic(&args, DBUS_TYPE_STRING,
                                        &property_name)) {
        fprintf(stderr, "Out Of Memory!\n");
        exit(1);
    }

    DBusPendingCall *pending;

    // send message and get a handle for a reply
    if (!dbus_connection_send_with_reply(conn, msg, &pending,
                                         -1)) { // -1 is default timeout
        fprintf(stderr, "Out Of Memory!\n");
        exit(1);
    }
    if (NULL == pending) {
        fprintf(stderr, "Pending Call Null\n");
        exit(1);
    }
    dbus_connection_flush(conn);

    // free message
    dbus_message_unref(msg);

    // block until we recieve a reply
    dbus_pending_call_block(pending);

    // get the reply message
    msg = dbus_pending_call_steal_reply(pending);
    if (NULL == msg) {
        fprintf(stderr, "Reply Null\n");
        exit(1);
    }

    const char *msg_err_name = dbus_message_get_error_name(msg);
    if (msg_err_name) {
        printf("Error sending message: (%s)\n", msg_err_name);
        exit(1);
    }

    // free the pending message handle
    dbus_pending_call_unref(pending);

    // parse the reply
    dbus_int64_t position;
    DBusMessageIter sub;
    if (!dbus_message_iter_init(msg, &args))
        fprintf(stderr, "Message has no arguments!\n");
    else
        dbus_message_iter_recurse(&args, &sub);

    dbus_message_iter_get_basic(&sub, &position);
    printf("Reply: %ld\n", position);

    // free reply
    dbus_message_unref(msg);
    // dbus_connection_unref(conn);
}

// /**
//  * Listens for signals on the bus
//  */
// void receive()
// {
//    DBusMessage* msg;
//    DBusMessageIter args;
//    DBusConnection* conn;
//    DBusError err;
//    int ret;
//    char* sigvalue;
//
//    printf("Listening for signals\n");
//
//    // initialise the errors
//    dbus_error_init(&err);
//
//    // connect to the bus and check for errors
//    conn = dbus_bus_get(DBUS_BUS_SESSION, &err);
//    if (dbus_error_is_set(&err)) {
//       fprintf(stderr, "Connection Error (%s)\n", err.message);
//       dbus_error_free(&err);
//    }
//    if (NULL == conn) {
//       exit(1);
//    }
//
//    // request our name on the bus and check for errors
//    ret = dbus_bus_request_name(conn, "test.signal.sink", DBUS_NAME_FLAG_REPLACE_EXISTING , &err);
//    if (dbus_error_is_set(&err)) {
//       fprintf(stderr, "Name Error (%s)\n", err.message);
//       dbus_error_free(&err);
//    }
//    if (DBUS_REQUEST_NAME_REPLY_PRIMARY_OWNER != ret) {
//       exit(1);
//    }
//
//    // add a rule for which messages we want to see
//    dbus_bus_add_match(conn, "type='signal',interface='test.signal.Type'", &err); // see signals from the given interface
//    dbus_connection_flush(conn);
//    if (dbus_error_is_set(&err)) {
//       fprintf(stderr, "Match Error (%s)\n", err.message);
//       exit(1);
//    }
//    printf("Match rule sent\n");
//
//    // loop listening for signals being emmitted
//    while (true) {
//
//       // non blocking read of the next available message
//       dbus_connection_read_write(conn, 0);
//       msg = dbus_connection_pop_message(conn);
//
//       // loop again if we haven't read a message
//       if (NULL == msg) {
//          sleep(1);
//          continue;
//       }
//
//       // check if the message is a signal from the correct interface and with the correct name
//       if (dbus_message_is_signal(msg, "test.signal.Type", "Test")) {
//
//          // read the parameters
//          if (!dbus_message_iter_init(msg, &args))
//             fprintf(stderr, "Message Has No Parameters\n");
//          else if (DBUS_TYPE_STRING != dbus_message_iter_get_arg_type(&args))
//             fprintf(stderr, "Argument is not string!\n");
//          else
//             dbus_message_iter_get_basic(&args, &sigvalue);
//
//          printf("Got Signal with value %s\n", sigvalue);
//       }
//
//       // free the message
//       dbus_message_unref(msg);
//    }
//    // close the connection
//    dbus_connection_close(conn);
// }

// int main(int argc, char** argv)
int main(void)
{
    DBusConnection *conn;
    DBusError err;

    // initialiset the errors
    dbus_error_init(&err);

    // connect to the session bus and check for errors
    conn = dbus_bus_get(DBUS_BUS_SESSION, &err);
    if (dbus_error_is_set(&err)) {
        fprintf(stderr, "Connection Error (%s)\n", err.message);
        dbus_error_free(&err);
    }
    if (NULL == conn) {
        exit(1);
    }

    // request our name on the bus
    int ret = dbus_bus_request_name(conn, "user.BarScripts",
                                    DBUS_NAME_FLAG_REPLACE_EXISTING, &err);
    if (dbus_error_is_set(&err)) {
        fprintf(stderr, "Name Error (%s)\n", err.message);
        dbus_error_free(&err);
    }
    if (DBUS_REQUEST_NAME_REPLY_PRIMARY_OWNER != ret) {
        exit(1);
    }

    get_all_bus_names(conn);

//     struct timespec time;
//     clock_gettime(CLOCK_REALTIME, &time);
// #define PADDING "000000000"
// #define PADDING_LEN 9
//     long previous_sec = time.tv_sec;
//     long previous_nano = time.tv_nsec;
//     while (1) {
//         clock_gettime(CLOCK_REALTIME, &time);
//         long current_sec = time.tv_sec;
//         long current_nano = time.tv_nsec;
//         long diff_sec = current_sec - previous_sec;
//         long diff_nano;
//         if (diff_sec > 0) {
//             diff_nano = (current_nano + 1000000000) - previous_nano;
//         } else {
//             diff_nano = current_nano - previous_nano;
//         }
//         previous_sec = current_sec;
//         previous_nano = current_nano;
//         int diff_nano_len = ((int)floorl(log10l((long double)diff_nano))) + 1;
//         int pad_len = PADDING_LEN - diff_nano_len;
//         if (pad_len < 0) {
//             pad_len = 0;
//         }
//         fprintf(stderr, "%ld.%.*s%ld\n", diff_sec, pad_len, PADDING, diff_nano);
//     }
    // if (2 > argc) {
    //    printf ("Usage: dbus-example [send|receive|listen|query] [<param>]\n");
    //    return 1;
    // }
    // char* param = "no param";
    // if (3 >= argc && NULL != argv[2]) param = argv[2];
    // if (0 == strcmp(argv[1], "send"))
    //    sendsignal(param);
    // else if (0 == strcmp(argv[1], "receive"))
    //    receive();
    // else if (0 == strcmp(argv[1], "listen"))
    //    listen();
    // else if (0 == strcmp(argv[1], "query"))
    //    query(param);
    // else {
    //    printf ("Syntax: dbus-example [send|receive|listen|query] [<param>]\n");
    //    return 1;
    // }
}
