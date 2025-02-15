"use client";
import React, { useEffect, useState } from "react";

interface FileItem {
  filename: string;
  size?: number;
  createdAt?: string;
  // Additional fields can be added if needed, e.g. size, createdAt, etc.
}

export default function FileList() {
  const [files, setFiles] = useState<FileItem[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  let listFilesApi = (process.env.FILE_SERVER_API || "") + "/api/files";
  let downloadApi = (process.env.FILE_SERVER_API || "") + "/api/download";

  async function fetchFiles() {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(listFilesApi); // Adjust this URL to match your API endpoint.
      if (!res.ok) {
        throw new Error(`Error fetching files: ${res.statusText}`);
      }
      const jsonData = await res.json();
      const data = jsonData
        .map((item: any) => ({
          filename: item.filename,
          size: item.size,
          createdAt: item.created_at,
          // Add more fields here if needed
        }));
      setFiles(data);
    } catch (err: any) {
      setError(err.message || "Unknown error");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    fetchFiles();
  }, []);

  return (
    <div>
      <p>Click on a file to download:</p>
      <hr />
      {loading && <p>Loading files...</p>}
      {error && <p style={{ color: "red" }}>Error: {error}</p>}
      {files.length === 0 && !loading ? (
        <p>No files uploaded yet.</p>
      ) : (
        <table className="w-full text-sm text-left rtl:text-right text-gray-500 dark:text-gray-400">
          <thead className="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
            <tr>
              <th scope="col" className="px-6 py-3">
                File Name
              </th>
              <th scope="col" className="px-6 py-3">
                Size
              </th>
              <th scope="col" className="px-6 py-3">
                Created At
              </th>
            </tr>
          </thead>
          <tbody>
            {files.map((file) => (
              <tr
                key={file.filename}
                className="bg-white border-b dark:bg-gray-800 dark:border-gray-700 border-gray-200"
              >
                <td
                  scope="row"
                  className="px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white"
                >
                  <a href={`${downloadApi}/${file.filename}`}>
                    {file.filename}
                  </a>
                </td>
                <td
                  scope="row"
                  className="px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white"
                >
                  {file.size
                    ? `${(file.size / 1024 / 1024).toLocaleString(undefined, {
                        minimumFractionDigits: 2,
                        maximumFractionDigits: 2,
                      })} MB`
                    : "-"}
                </td>
                <td
                  scope="row"
                  className="px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white"
                >
                  {file.createdAt
                    ? new Date(file.createdAt).toLocaleString()
                    : "-"}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <button
        onClick={fetchFiles}
        style={{
          marginTop: "1rem",
          textDecoration: "underline",
          color: "yellow",
        }}
      >
        Refresh List
      </button>
    </div>
  );
}
