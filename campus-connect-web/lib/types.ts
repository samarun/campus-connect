export type Role = "admin" | "operator" | "main" | "campus-b" | "campus-c";

export type Mode = "greeting" | "program";

export type SessionUser = {
  role: Role;
  name: string;
};

export type CampusKey = "main" | "b" | "c";

export type ReceiverKey = "main" | "campus-b" | "campus-c" | "grid";

export type RoomName = "campus-greeting" | "campus-program" | "campus-backstage";

export type TenantSettings = {
  platformName: string;
  organizationName: string;
  primarySiteName: string;
  remoteSiteAName: string;
  remoteSiteBName: string;
  greetingModeName: string;
  programModeName: string;
};
