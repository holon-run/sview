export interface User {
  id: string;
}

type UserId = string;

enum Mode {
  Fast,
  Slow,
}

export class Service {
  async load(id: UserId): Promise<User> {
    return { id };
  }
}
