// Initialize the catalog downloading queue
console.log("Initializing catalog downloading queue");
sessionStorage.setItem("catalog-downloading", JSON.stringify([]));

// Update the download queue (add or remove an item)
function updateQueue(path, action) {
  console.log(`Updating queue. Action: ${action}, Path: ${path}`);
  // Retrieve the current queue from sessionStorage
  let queue = JSON.parse(sessionStorage.getItem("catalog-downloading"));

  // Add or remove the path from the queue based on the action
  if (action === "add") {
    if (!queue.includes(path)) {
      queue.push(path);
    }
  } else if (action === "remove") {
    queue = queue.filter((item) => item !== path);
  }

  // Update the sessionStorage with the modified queue
  sessionStorage.setItem("catalog-downloading", JSON.stringify(queue));
  console.log("Updated queue: ", queue);
}

// Trigger a download for the given blob and filename
function downloadBlob(blob, filename) {
  console.log("Downloading blob with filename: ", filename);
  // Create a URL for the blob object
  const blobUrl = URL.createObjectURL(blob);
  // Create an anchor element to trigger the download
  const a = document.createElement("a");

  a.href = blobUrl;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  // Revoke the object URL to free memory
  URL.revokeObjectURL(blobUrl);
  console.log("Download completed for: ", filename);
}

// Handle the download process for a given path and button
function handleDownload(path, button) {
  console.log("Starting download for path: ", path);

  // If the file is already being downloaded, return early
  const queue = JSON.parse(sessionStorage.getItem("catalog-downloading"));
  if (queue.includes(path)) {
    console.log("File is already being downloaded: ", path);
    return;
  }

  // Set loading state for the button
  const originalText = button.innerHTML;
  console.log("Setting button to loading state for path: ", path);
  button.innerHTML = '<img src="../../assets/loading.gif" alt="Loading..." />';
  button.disabled = true;
  // Add the path to the download queue
  updateQueue(path, "add");

  // Fetch the file and trigger the download
  fetch(path)
    .then((response) => {
      console.log("Fetch response received for path: ", path);
      // Check if the response is successful
      if (!response.ok) {
        console.error("Network response was not ok for path: ", path);
        throw new Error("Network response was not ok");
      }
      // Return the response as a blob
      return response.blob();
    })
    .then((blob) => {
      console.log("Blob received for path: ", path);
      // Extract the filename from the path
      const fileName = path.split("/").pop();
      // Trigger the download of the blob
      downloadBlob(blob, fileName);
      // Remove the path from the download queue
      updateQueue(path, "remove");
      // Restore the original button state
      button.innerHTML = originalText;
      button.disabled = false;
      console.log("Download process completed for path: ", path);
    })
    .catch((error) => {
      // Handle any errors that occur during the download
      console.error("Error during download for path: ", path, error);
      updateQueue(path, "remove");
      button.innerHTML = originalText;
      button.disabled = false;
    });
}

// Wait for an element to be available in the DOM by its ID
function waitForElementById(id) {
  console.log("Waiting for element with ID: ", id);
  return new Promise((resolve) => {
    // Poll the DOM every 100ms to check if the element is available`zzF
    const interval = setInterval(() => {
      if (document.getElementById(id)) {
        console.log("Element found with ID: ", id);
        clearInterval(interval);
        resolve();
      }
    }, 100);
  });
}

// Load elements and attach download handlers
async function loader() {
  console.log("Starting loader");
  // Wait for specific elements to be available in the DOM
  await waitForElementById("catalog");
  await waitForElementById("map-asset-vcc");

  // Attach download handlers to buttons after a short delay
  setTimeout(() => {
    console.log("Attaching download handlers to buttons");
    document.querySelectorAll("button[data-href]").forEach((element) => {
      if (!element.hasAttribute("data-processed")) {
        console.log(
          "Attaching handler to button with data-href: ",
          element.getAttribute("data-href"),
        );
        element.setAttribute("data-processed", "true");
        // Set the onclick handler for the button to handle the download
        element.onclick = () =>
          handleDownload(element.getAttribute("data-href"), element);
      }
    });
  }, 250);
}

// Run the loader function and set data attributes when ready
loader().then(() => {
  console.log("Loader completed");
  const catalogElement = document.getElementById("catalog");
  const mapAssetElement = document.getElementById("map-asset-vcc");

  // Set attributes for the catalog element if it exists
  if (catalogElement) {
    console.log("Setting attributes for catalog element");
    catalogElement.setAttribute("data-processed", "true");
    catalogElement.setAttribute("data-trunk", "true");
  }

  // Set attributes for the map-asset element if it exists
  if (mapAssetElement) {
    console.log("Setting attributes for map-asset element");
    mapAssetElement.setAttribute("data-trunk", "true");
  }
});
