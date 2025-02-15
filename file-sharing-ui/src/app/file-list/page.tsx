import React from "react";
import FileList from "../components/FileList";
import Link from "next/link";

export default function FileListPage() {
  return (
    <div className="grid grid-rows-[20px_1fr_20px] items-center justify-items-center min-h-screen p-8 pb-20 gap-16 sm:p-20 font-[family-name:var(--font-geist-sans)]">
      <main className="flex flex-col gap-8 row-start-2 items-center sm:items-start">
        <h1>All Uploaded Files</h1>
        <FileList />
      </main>
      <footer className="row-start-3 flex gap-6 flex-wrap items-center justify-center">
        <Link href="/" passHref className="font-medium text-blue-600 dark:text-blue-500 hover:underline" >
          Upload More Files
        </Link>
      </footer>
    </div>
  );
}
