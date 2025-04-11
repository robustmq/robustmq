# How to Build the Documentation Environment

RobustMQ uses [VitePress](https://vitepress.dev/) to build its documentation system. If you need to modify the
configuration, you can refer to the [VitePress documentation](https://vitepress.dev/guide/getting-started) to help
improve the documentation build for RobustMQ.

## Mac / Windows

### Prerequisites

You need to have the `node` environment installed.

- macOS users can install Node.js via `brew`:

```shell
brew install node
```

- Windows users can install Node.js from the official [Node.js](https://nodejs.org/en/download/) website.

### Steps

1. Install the packages required by `VitePress` using the following command:

```shell
npm install
```

2. Start local development with the following command:

```shell
npm run docs:dev
```

3. Open the local link, and the final result should look like this:

![image](../../../images/Build-Doc-Env-1.png)
