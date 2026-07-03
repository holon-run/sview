#include <string>

namespace demo {
class Client {
public:
    Client() = default;
    void fetch(int id);

private:
    std::string name_;
};

void Client::fetch(int id) {}

enum class Mode { Fast, Slow };

int add(int a, int b) { return a + b; }
}
