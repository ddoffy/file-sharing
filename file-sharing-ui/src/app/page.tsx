"use client";
import { useState } from "react";
import Link from "next/link";
import DragDropUploader from "./components/DragDropUploader";

export default function Home() {
  const [progresses, setProgresses] = useState([]);
  const [statuses, setStatuses] = useState([]);
  const uploadApi =  (process.env.FILE_SERVER_API || "") + "/api/upload";

  const handleFiles = (files: FileList) => {
    // handle the files (e.g. upload to a server)
    if (!files.length) {
      return;
    }

    Object.keys(files).forEach((key) => {
      handleUpload(files[key]);
    });
  };

  const handleUpload = async (file) => {
    const formData = new FormData();
    formData.append("file", file);

    const xhr = new XMLHttpRequest();

    xhr.open("POST", uploadApi, true);

    xhr.upload.onprogress = (event) => {
      if (event.lengthComputable) {
        const percentage = (event.loaded / event.total) * 100;
        setProgresses((prev) => {
          const newProgresses = [...prev];
          newProgresses[file.name] = percentage;
          return newProgresses;
        });
      }
    };

    xhr.upload.onload = () => {
      if (xhr.status === 200) {
        setStatuses((prev) => {
          const newStatuses = [...prev];
          newStatuses[file.name] = "Upload successful";
          return newStatuses;
        });
      } else {
        setStatuses((prev) => {
          const newStatuses = [...prev];
          newStatuses[file.name] = `Upload failed: ${xhr.status}`;
          return newStatuses;
        });
      }
    };

    xhr.onerror = () => {
      setStatuses((prev) => {
        const newStatuses = [...prev];
        newStatuses[file.name] = "Upload failed";
        return newStatuses;
      });
    };

    xhr.send(formData);
  };

  return (
    <div className="w-full overflow-x-auto" >
      <main className="min-w-full text-sm text-left rtl:text-right text-gray-500 dark:text-gray-400">
        <DragDropUploader onFilesSelected={handleFiles} />
      </main>
      <footer className="row-start-3 flex gap-6 flex-wrap items-center justify-center">
        <div>
          {statuses &&
            progresses &&
            Object.keys(progresses).map((key) => (
              <div key={key} className="flex items-center gap-4">
                <span>{key}</span>
                <div className="w-64 h-4 bg-gray-300 rounded-full">
                  <div
                    style={{ width: `${progresses[key]}%` }}
                    className="h-full bg-green-500 rounded-full"
                  ></div>
                </div>
              </div>
            ))}
        </div>
        <div>
          <Link href="/file-list" passHref className="font-medium text-blue-600 dark:text-blue-500 hover:underline">
            View Uploaded Files
          </Link>
        </div>
      </footer>
    </div>
  );
}
