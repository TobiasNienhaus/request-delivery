import {
  matchesContentType,
  contentTypeFrom,
  isContentType,
  ContentType,
} from '@ganbarodigital/ts-lib-mediatype/lib/v1';

export enum BodyType {
  UrlForm = 'url-form',
  MultiPartForm = 'multi-part-form',
  Json = 'json',
  Xml = 'xml',
  Binary = 'binary',
  Unknown = 'unknown',
}

const PLAIN_TYPES: ContentType[] = ['text/plain'].map((v) =>
  contentTypeFrom(v)
);
const JSON_TYPES: ContentType[] = [
  'application/json',
  'application/ld+json',
].map((v) => contentTypeFrom(v));
// TODO csv
const XML_TYPES: ContentType[] = ['application/xml', 'text/html'].map((v) =>
  contentTypeFrom(v)
);
const CODE_TYPES: ContentType[] = ['text/javascript'].map((v) =>
  contentTypeFrom(v)
);
const FORM_TYPES: ContentType[] = ['application/x-www-form-urlencoded'].map(
  (v) => contentTypeFrom(v)
);
const MULTI_PART_FORM_TYPES: ContentType[] = ['multipart/form-data'].map((v) =>
  contentTypeFrom(v)
);

const SHOW_CODE_TYPES: ContentType[] = [
  ...PLAIN_TYPES,
  ...JSON_TYPES,
  ...XML_TYPES,
  ...CODE_TYPES,
];

const SHOW_INTERACTIVE_TYPES: ContentType[] = [...JSON_TYPES, ...XML_TYPES];

export function canShowCode(ct: string): boolean {
  return (
    isContentType(ct) &&
    matchesContentType(contentTypeFrom(ct), SHOW_CODE_TYPES)
  );
}

export function canShowInteractive(ct: string): boolean {
  return (
    isContentType(ct) &&
    matchesContentType(contentTypeFrom(ct), SHOW_INTERACTIVE_TYPES)
  );
}

export function getBodyType(contentType: string): BodyType {
  if (!isContentType(contentType)) {
    return BodyType.Unknown;
  }
  let ct = contentTypeFrom(contentType);
  if (matchesContentType(ct, JSON_TYPES)) {
    return BodyType.Json;
  }
  if (matchesContentType(ct, FORM_TYPES)) {
    return BodyType.UrlForm;
  }
  if (matchesContentType(ct, FORM_TYPES)) {
    return BodyType.UrlForm;
  }
  if (matchesContentType(ct, MULTI_PART_FORM_TYPES)) {
    return BodyType.MultiPartForm;
  }
  if (matchesContentType(ct, XML_TYPES)) {
    return BodyType.Xml;
  }
  return BodyType.Unknown;
}
