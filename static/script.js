document.getElementById("shorten-form").addEventListener(
  "submit",
  function(event) {
    event.preventDefault();

    const url = document.getElementById("url-input").value;
    const slug = document.getElementById("slug-input").value;
    const responseMessage = document.getElementById("response-message");

    const payload = {
      redirect: url,
    };

    if (slug) {
      payload.link = slug;
    }

    fetch("/new", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(payload),
    })
      .then((response) => response.json())
      .then((data) => {
        if (data.message) {
          responseMessage.innerHTML =
            'Link created at <a href="https://link.taranathan.com/' +
            data.message + '">https://link.taranathan.com/' + data.message +
            "</a>";
          console.log(responseMessage);
        } else if (data.error) {
          responseMessage.textContent = "Error: " + data.error;
        } else {
          responseMessage.textContent = "An unexpected response was received.";
        }
      })
      .catch((error) => {
        console.error("Error:", error);
        responseMessage.textContent =
          "An error occurred while communicating with the server.";
      });
  },
);
