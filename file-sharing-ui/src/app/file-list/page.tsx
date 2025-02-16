import React from "react";
import FileList from "../components/FileList";
import Link from "next/link";

export default function FileListPage() {
  return (
    <div className="w-full overflow-x-auto">
      <main className="min-w-full text-sm text-left rtl:text-right text-gray-500 dark:text-gray-400">
        <FileList />
      </main>
      <footer className="row-start-3 flex gap-6 flex-wrap items-center justify-center">
        <Link
          href="/"
          passHref
          className="font-medium text-blue-600 dark:text-blue-500 hover:underline"
        >
          Upload More Files
        </Link>
      </footer>
    </div>
  );
}
