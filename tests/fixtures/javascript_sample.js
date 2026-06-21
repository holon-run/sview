export function loadUser(id) {
  return id;
}

export class Client {
  fetch() {
    return loadUser(1);
  }
}

const helper = () => true;
