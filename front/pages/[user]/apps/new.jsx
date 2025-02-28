import AppLayout from "../../../components/app/AppLayout";
import MainTab from "../../../components/profile/MainTab";
import { unstable_getServerSession } from "next-auth/next";
import { authOptions } from "../../api/auth/[...nextauth]";
import { Button } from "../../../components/Button";
import { useSession } from "next-auth/react";
import React, { useState, useEffect, useRef } from "react";
import { classNames } from "../../../lib/utils";
import { ChevronRightIcon } from "@heroicons/react/20/solid";

const { URL } = process.env;

export default function New({ apps }) {
  const { data: session } = useSession();

  const [disable, setDisabled] = useState(true);

  const [appName, setAppName] = useState("");
  const [appNameError, setAppNameError] = useState(null);

  const [appDescription, setAppDescription] = useState("");
  const [appVisibility, setAppVisibility] = useState("public");

  const formValidation = () => {
    if (appName.length == 0) {
      setAppNameError(null);
      return false;
    } else if (!appName.match(/^[a-zA-Z0-9\._\-]+$/)) {
      setAppNameError(
        "App name must only contain letters, numbers, and the characters `._-`"
      );
      return false;
    } else {
      setAppNameError(null);
      return true;
    }
  };

  useEffect(() => {
    setDisabled(!formValidation());
  }, [appName]);

  return (
    <AppLayout>
      <div className="flex flex-col">
        <div className="flex flex-initial mt-2">
          <MainTab current_tab="Apps" />
        </div>
        <div className="flex flex-1">
          <div className="mx-auto max-w-4xl px-4 sm:px-6 lg:px-8">
            <form
              action={`/api/apps/${session.user.username}`}
              method="POST"
              className="space-y-8 divide-y divide-gray-200 mt-8"
            >
              <div className="space-y-8 divide-y divide-gray-200">
                <div>
                  <h3 className="text-base font-medium leading-6 text-gray-900">
                    Create a new App
                  </h3>
                  <p className="mt-1 text-sm text-gray-500">
                    An app consists of its specification (defining chained
                    interactions with models and external services), datasets to
                    run the app or few-shot prompt models. Everything is
                    automatically versioned and stored.
                  </p>
                </div>
                <div>
                  <div className="mt-6 grid grid-cols-1 gap-y-6 gap-x-4 sm:grid-cols-6">
                    <div className="sm:col-span-3">
                      <label
                        htmlFor="appName"
                        className="block text-sm font-medium text-gray-700"
                      >
                        App name
                      </label>
                      <div className="mt-1 flex rounded-md shadow-sm">
                        <span className="inline-flex items-center rounded-l-md border border-r-0 border-gray-300 bg-gray-50 pl-3 pr-1 text-gray-500 sm:text-sm">
                          {session.user.username}
                          <ChevronRightIcon
                            className="h-5 w-5 flex-shrink-0 text-gray-400 pt-0.5"
                            aria-hidden="true"
                          />
                        </span>
                        <input
                          type="text"
                          name="name"
                          id="appName"
                          className={classNames(
                            "block w-full min-w-0 flex-1 rounded-none rounded-r-md sm:text-sm",
                            appNameError
                              ? "border-gray-300 focus:border-red-500 border-red-500 focus:ring-red-500"
                              : "border-gray-300 focus:border-violet-500 focus:ring-violet-500"
                          )}
                          value={appName}
                          onChange={(e) => setAppName(e.target.value)}
                        />
                      </div>
                      <p className="mt-2 text-sm text-gray-500">
                        Think GitHub repository names, short and memorable.
                      </p>
                    </div>

                    <div className="sm:col-span-6">
                      <div className="flex justify-between">
                        <label
                          htmlFor="appDescription"
                          className="block text-sm font-medium text-gray-700"
                        >
                          Description
                        </label>
                        <div className="font-normal text-gray-400 text-sm">
                          optional
                        </div>
                      </div>
                      <div className="mt-1 flex rounded-md shadow-sm">
                        <input
                          type="text"
                          name="description"
                          id="appDescription"
                          className="block w-full min-w-0 flex-1 rounded-md border-gray-300 focus:border-violet-500 focus:ring-violet-500 sm:text-sm"
                          value={appDescription}
                          onChange={(e) => setAppDescription(e.target.value)}
                        />
                      </div>
                      <p className="mt-2 text-sm text-gray-500">
                        A good description will help others discover and
                        understand the purpose of your app.
                      </p>
                    </div>

                    <div className="sm:col-span-6">
                      <fieldset className="mt-2">
                        <legend className="contents text-sm font-medium text-gray-700">
                          Visibility
                        </legend>
                        <div className="mt-4 space-y-4">
                          <div className="flex items-center">
                            <input
                              id="appVisibilityPublic"
                              name="visibility"
                              type="radio"
                              className="h-4 w-4 border-gray-300 text-violet-600 focus:ring-violet-500"
                              value="public"
                              checked={appVisibility == "public"}
                              onChange={(e) => {
                                if (e.target.value != appVisibility) {
                                  setAppVisibility(e.target.value);
                                }
                              }}
                            />
                            <label
                              htmlFor="app-visibility-public"
                              className="ml-3 block text-sm font-medium text-gray-700"
                            >
                              Public
                              <p className="mt-0 text-sm font-normal text-gray-500">
                                Anyone on the Internet can see the app. You
                                choose who can edit.
                              </p>
                            </label>
                          </div>
                          <div className="flex items-center">
                            <input
                              id="appVisibilityPrivate"
                              name="visibility"
                              type="radio"
                              value="private"
                              className="h-4 w-4 border-gray-300 text-violet-600 focus:ring-violet-500"
                              checked={appVisibility == "private"}
                              onChange={(e) => {
                                if (e.target.value != appVisibility) {
                                  setAppVisibility(e.target.value);
                                }
                              }}
                            />
                            <label
                              htmlFor="app-visibility-private"
                              className="ml-3 block text-sm font-medium text-gray-700"
                            >
                              Private
                              <p className="mt-0 text-sm font-normal text-gray-500">
                                You choose who can see and edit the app.
                              </p>
                            </label>
                          </div>
                        </div>
                      </fieldset>
                    </div>
                  </div>
                </div>
              </div>

              <div className="pt-6">
                <div className="flex">
                  <Button
                    disabled={disable}
                    type="submit"
                    // onClick={() => handleSubmit()}
                  >
                    Create
                  </Button>
                </div>
              </div>
            </form>
          </div>
        </div>
      </div>
    </AppLayout>
  );
}

export async function getServerSideProps(context) {
  const session = await unstable_getServerSession(
    context.req,
    context.res,
    authOptions
  );

  if (!session) {
    return {
      redirect: {
        destination: `/`,
        permanent: false,
      },
    };
  }

  if (context.query.user != session.user.username) {
    return {
      redirect: {
        destination: `/`,
        permanent: false,
      },
    };
  }

  const [appsRes] = await Promise.all([
    fetch(`${URL}/api/apps/${session.user.username}`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Cookie: context.req.headers.cookie,
      },
    }),
  ]);

  if (appsRes.status === 404) {
    return {
      notFound: true,
    };
  }

  const [apps] = await Promise.all([appsRes.json()]);

  return {
    props: { session, apps: apps.apps },
  };
}
