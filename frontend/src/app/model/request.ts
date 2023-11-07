export type Method =
  | 'GET'
  | 'PUT'
  | 'POST'
  | 'DELETE'
  | 'OPTIONS'
  | 'HEAD'
  | 'TRACE'
  | 'CONNECT'
  | 'PATCH';

export interface RequestData {
  method: Method;
  contentType: string | null;
  body: string | null;
  complete: boolean | null;
  headers: Record<string, string[]>;
  cookies: Record<string, string[]>;
  uri: string;
  remote: {
    host: string | null;
    remoteIp: string | null;
    headerIp: string | null;
    clientIp: string | null;
  };
  time: Date;
}

export let DEFAULT_REQUEST: RequestData = {
  method: 'POST',
  contentType: 'application/json',
  body: '{\r\n    "Hallo": "Hallo",\r\n    "1": 1,\r\n    "nested": {\r\n        "nested": "A",\r\n        "1:1": 1\r\n    },\r\n    "arr": [\r\n        "A",\r\n        1,\r\n        "sldf",\r\n        {\r\n            "a": "a",\r\n            "b": "c"\r\n        }\r\n    ]\r\n}',
  complete: true,
  headers: {
    'accept-encoding': ['gzip, deflate, br'],
    cookie: ['Cookie_1=value2'],
    'x-real-ip': ['127.0.0.2'],
    'postman-token': ['980fa8c7-ec4f-49b1-9dcc-56cd63022be5'],
    acc: ['ass', 'asdadw', 'Aed3reww', 'sdfkjeyhfkjdsfdsfvcx'],
    'content-length': ['238'],
    'x-forwarded-for': ['23.212.121.1'],
    'user-agent': ['PostmanRuntime/7.34.0'],
    'content-type': ['application/json'],
    accept: ['application/atom+xml', 'aa'],
    connection: ['keep-alive'],
    'cache-control': ['no-cache'],
    host: ['localhost:18234'],
  },
  cookies: {
    Cookie_1: ['value2'],
  },
  uri: '/send/cb0688e1-cd7c-4d02-a5c2-03608b97593c?test=21232',
  remote: {
    host: 'localhost:18234',
    remoteIp: '127.0.0.1:55017',
    headerIp: '127.0.0.2',
    clientIp: '127.0.0.2',
  },
  time: new Date(),
};
