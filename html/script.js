let Books, Book;

const formTitle = document.getElementById("form-title");
const formPages = document.getElementById("form-pages");
const formStatus = document.getElementById("form-status");
const formButton = document.getElementById("form-button");

const booksPanel = document.getElementById("books");
const paginationInput = document.getElementById("pagination-input");
const paginationPages = document.getElementById("pages-count");

/**
 * @typedef Book
 * @property {string} title
 * @property {number} pages
 */

/**
 * @typedef Books
 * @property {Book[]} books
 */

/**
 * Initialize Protobuf
 */
async function init() {
  return new Promise(function(resolve, reject) {
    protobuf.load("book.proto", function(err, root) {
      if (err) {
        throw err;
      }

      Books = root.lookupType("book.Books");
      Book = root.lookupType("book.Book");

      resolve();
    });
  });
}

/**
  * @param {ArrayBuffer} buffer
  * @returns {Books} results
  */
function decodeBooks(buffer) {
  return Books.decode(new Uint8Array(buffer));
}

/**
  * @param {ArrayBuffer} buffer
  * @returns {Book} results
  */
function decodeBook(buffer) {
  return Book.decode(new Uint8Array(buffer));
}

function onUpsert(e, isButton) {
  if (!isButton) {
    if (e.key !== "Enter") {
      return;
    }
  }

  if (formTitle.value.length === 0 || formTitle.value < 0 || formPages.value.lendth === 0 || formPages.value < 0) {
    alert("Title or Pages must be more than 0.");
    return;
  }

  formButton.disabled = true;
  fetch(`/api/book/title-${formTitle.value}/${formPages.value}`, {
    method: "PUT"
  })
    .then((response) => {
      if (response.status !== 200 && response.status !== 304) {
        throw Error(response.statusText);
      }
      return { status: response.status };
    })
    .then(({ status }) => {
      if (status === 200) {
        formStatus.innerText = "OK";
      } else {
        formStatus.innerText = "Not Modified";
      }
    })
    .catch((error) => {
      console.error(error);
      formStatus.innerText = "Error";
    })
    .finally(() => {
      formButton.disabled = false;
    });
}

function loadBooks() {
  const query = new URLSearchParams(window.location.search);

  fetch("/api/books/pagination", {
    method: "GET"
  })
    .then((response) => {
      return response.text();
    })
    .then((text) => {
      const pages = Number.parseInt(text);
      paginationPages.innerText = pages;

      let i = query.get("i");

      if (!i) {
        i = 1;
      } else if (i > pages) {
        i = pages;
        query.set("i", pages);
        window.history.replaceState(null, null, `?${query.toString()}`);
      }

      fetch(`/api/books?i=${query.get("i")}`, {
        method: "GET",
      })
        .then((response) => {
          return response.arrayBuffer();
        })
        .then((buffer) => {
          const results = decodeBooks(buffer);
          booksPanel.innerHTML = "";
          results.books.forEach((book) => {
            const element = document.createElement("p");
            const elementTitle = document.createElement("span");
            const elementPages = document.createElement("span");

            elementTitle.innerText = `title: ${book.title}`;
            elementPages.innerText = `pages: ${book.pages}`;

            element.appendChild(elementTitle);
            element.appendChild(document.createElement("br"));
            element.appendChild(elementPages);
            booksPanel.appendChild(element);

            paginationInput.value = Number.parseInt(i);
          });
        })
        .catch((error) => {
          console.log(error);
        });
    })
    .catch((error) => {
      console.error(error);
    });
}

function onPagination(isNext, element, event) {
  const query = new URLSearchParams(window.location.search);

  if (element !== undefined) {
    if (event.key === "Enter") {
      query.set("i", element.value);
    } else {
      return;
    }
  }

  let i = query.get("i");
  if (!i) {
    i = 1;
  }

  if (element === undefined && isNext === true) {
    console.log("next");
    i = Number.parseInt(i) + 1;
    query.set("i", i);
  } else if (element === undefined && isNext === false) {
    console.log("prev");
    i = Number.parseInt(i) - 1;
    query.set("i", i);
  }

  if (i <= 1) {
    query.set("i", 1);
  }

  window.history.replaceState(null, null, `?${query.toString()}`);

  loadBooks();
}

async function onSort() {
  await fetch("/api/books/sort", {
    method: "PATCH"
  });

  loadBooks();
}

// await fetch("/api/book/title-10000001", {
//   method: "GET"
// })
//   .then((response) => {
//     if (response.status !== 200) {
//       throw Error(response.statusText);
//     }
//     return response.arrayBuffer();
//   })
//   .then((buffer) => {
//     const result = decodeBook(buffer);
//   })
//   .catch((error) => {
//     console.log(error);
//   });

(async () => {
  await init();
  loadBooks();
})()
