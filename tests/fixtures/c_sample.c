#include <stdio.h>

struct Client {
    int id;
    const char *name;
};

enum Mode {
    MODE_FAST,
    MODE_SLOW,
};

static struct Client client_create(int id, const char *name) {
    return (struct Client){id, name};
}
