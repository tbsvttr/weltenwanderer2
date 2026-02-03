import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("weltenwanderer");
  const lspPath = config.get<string>("lspPath", "ww-lsp");

  const serverOptions: ServerOptions = {
    command: lspPath,
    args: [],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "ww" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.ww"),
    },
  };

  client = new LanguageClient(
    "weltenwanderer",
    "Weltenwanderer Language Server",
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
