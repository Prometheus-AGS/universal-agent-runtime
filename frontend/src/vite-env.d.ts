/// <reference types="vite/client" />

declare module "*.data?url" {
  const url: string;
  export default url;
}
