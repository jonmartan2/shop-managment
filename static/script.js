let currentItemId = null;
let uploadedImageUrl = null;
let allItems = []; // cache for search filtering

// Dark Mode Support
function initTheme() {
  const savedTheme = localStorage.getItem("theme");
  if (savedTheme === "dark") {
    document.body.classList.add("dark-mode");
    document.getElementById("themeToggle").textContent = "☀️";
  }
}

function toggleDarkMode() {
  const isDark = document.body.classList.toggle("dark-mode");
  localStorage.setItem("theme", isDark ? "dark" : "light");
  document.getElementById("themeToggle").textContent = isDark ? "☀️" : "🌙";
}

function toggleSearch() {
  const box = document.getElementById("searchBox");
  box.classList.toggle("visible");
  if (box.classList.contains("visible")) {
    box.focus();
  } else {
    box.value = "";
    renderItems(allItems);
  }
}

function filterItems() {
  const query = document.getElementById("searchBox").value.trim().toLowerCase();
  if (!query) {
    renderItems(allItems);
    return;
  }
  const filtered = allItems.filter(item =>
    item.name.toLowerCase().includes(query)
  );
  renderItems(filtered);
}

function openModal() {
  document.getElementById("modal").style.display = "flex";
  uploadedImageUrl = null;
  document.getElementById("imageUpload").value = "";
  document.getElementById("imagePreview").style.display = "none";
  document.getElementById("imagePreview").src = "";
  document.getElementById("uploadStatus").textContent = "";
  document.getElementById("itemName").value = "";
  document.getElementById("itemPrice").value = "";
  document.getElementById("itemQuantity").value = "10";
}

function closeModal() {
  document.getElementById("modal").style.display = "none";
}

function closeInfoModal() {
  document.getElementById("infoModal").style.display = "none";
}

async function handleImageUpload(input) {
  const file = input.files[0];
  if (!file) return;

  const status = document.getElementById("uploadStatus");
  status.textContent = "جارٍ الرفع...";
  status.style.color = "#888";

  const formData = new FormData();
  formData.append("image", file);

  try {
    const response = await fetch("/upload", {
      method: "POST",
      body: formData,
    });

    const result = await response.json();

    if (response.ok && result.url) {
      uploadedImageUrl = result.url;
      status.textContent = "✅ تم الرفع!";
      status.style.color = "#4caf50";

      const preview = document.getElementById("imagePreview");
      preview.src = result.url;
      preview.style.display = "block";
    } else {
      status.textContent = "❌ " + (result.error || "فشل الرفع");
      status.style.color = "#e53935";
      uploadedImageUrl = null;
    }
  } catch (error) {
    console.error("Upload error:", error);
    status.textContent = "❌ فشل الرفع";
    status.style.color = "#e53935";
    uploadedImageUrl = null;
  }
}

async function addItem() {
  const itemName = document.getElementById("itemName").value;
  const itemPrice = document.getElementById("itemPrice").value;
  const itemQuantity = document.getElementById("itemQuantity").value;

  if (!uploadedImageUrl || !itemName || !itemPrice || !itemQuantity) {
    alert("يرجى رفع صورة وملء جميع الحقول");
    return;
  }

  const newItem = {
    image: uploadedImageUrl,
    name: itemName,
    price: parseFloat(itemPrice),
    quantity: parseInt(itemQuantity),
  };

  try {
    const response = await fetch("/items", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(newItem),
    });

    if (response.ok) {
      await fetchAndRenderItems();
      closeModal();
    } else {
      alert("فشل إضافة المنتج");
    }
  } catch (error) {
    console.error("Error adding item:", error);
  }
}

function openInfoModal(item) {
  currentItemId = item.id;
  const infoContent = document.getElementById("infoContent");

  infoContent.innerHTML = `
    <img src="${item.image}" alt="${item.name}" onerror="this.src='https://via.placeholder.com/300x200?text=صورة+غير+متوفرة'">
    <h2>${item.name}</h2>
    <div class="info-price">$${item.price.toFixed(2)}</div>
    <div class="info-stats">
      <p>📦 الكمية: <span id="modalQuantity">${item.quantity}</span></p>
      <p>💰 المبيع: <span id="modalSold">${item.sold}</span></p>
      <p>💵 الإيرادات: $<span id="modalRevenue">${(item.sold * item.price).toFixed(2)}</span></p>
    </div>
    <div class="sell-actions">
      <input type="number" id="sellQuantity" class="sell-qty-input" value="1" min="1" max="${item.quantity}">
      <button class="sell-btn" onclick="performSell(${item.id})" ${item.quantity === 0 ? "disabled" : ""}>
        ${item.quantity === 0 ? "نفذ المخزون" : "تأكيد البيع 🛒"}
      </button>
    </div>
    <button class="close-info-btn" onclick="closeInfoModal()">إغلاق</button>
  `;

  document.getElementById("infoModal").style.display = "flex";
}

async function performSell(id) {
  const qtyInput = document.getElementById("sellQuantity");
  const quantity = parseInt(qtyInput.value);

  if (isNaN(quantity) || quantity <= 0) {
    alert("يرجى إدخال كمية صحيحة");
    return;
  }

  const item = allItems.find(i => i.id === id);
  if (!item) return;

  if (quantity > item.quantity) {
    alert("الكمية المطلوبة أكبر من المخزون المتوفر");
    return;
  }

  if (!confirm(`هل أنت متأكد من بيع ${quantity} من "${item.name}"؟`)) {
    return;
  }

  try {
    const response = await fetch(`/items/${id}/sell`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ quantity })
    });

    if (response.ok) {
      await fetchAndRenderItems();

      if (currentItemId === id) {
        const updatedItem = allItems.find(i => i.id === id);
        if (updatedItem) {
          document.getElementById("modalQuantity").textContent = updatedItem.quantity;
          document.getElementById("modalSold").textContent = updatedItem.sold;
          document.getElementById("modalRevenue").textContent = (updatedItem.sold * updatedItem.price).toFixed(2);

          const sellBtn = document.querySelector(".sell-btn");
          const qtyInputModal = document.getElementById("sellQuantity");
          qtyInputModal.max = updatedItem.quantity;
          if (updatedItem.quantity === 0) {
            sellBtn.textContent = "نفذ المخزون";
            sellBtn.disabled = true;
            qtyInputModal.disabled = true;
          }
        }
      }
    } else {
      const message = await response.text();
      alert(message);
    }
  } catch (error) {
    console.error("Error selling item:", error);
  }
}

async function fetchAndRenderItems() {
  try {
    const response = await fetch("/items");
    if (response.ok) {
      allItems = await response.json();
      filterItems(); // respect active search query
    } else {
      console.error("Failed to fetch items");
    }
  } catch (error) {
    console.error("Error fetching items:", error);
  }
}

function renderItems(items) {
  const grid = document.getElementById("grid");

  if (items.length === 0) {
    grid.innerHTML = '<div class="empty-message">لا توجد منتجات، اضغط إضافة لإضافة منتجات جديدة</div>';
    return;
  }

  grid.innerHTML = items.map(item => `
    <div class="card" onclick='openInfoModal(${JSON.stringify(item)})'>
      ${item.quantity === 0 ? '<div class="out-of-stock">نفذ المخزون</div>' : ""}
      <img src="${item.image}" alt="${item.name}" onerror="this.src='https://via.placeholder.com/300x200?text=صورة+غير+متوفرة'">
      <div class="card-content">
        <div class="card-title">${item.name}</div>
        <div class="card-price">$${item.price.toFixed(2)}</div>
        <div class="card-stats">
          <span class="quantity-badge">📦 ${item.quantity}</span>
          <span class="sold-badge">💰 ${item.sold} مباع</span>
        </div>
      </div>
    </div>
  `).join("");
}

// Close modal when clicking outside
window.onclick = function (event) {
  const modal = document.getElementById("modal");
  const infoModal = document.getElementById("infoModal");
  if (event.target === modal) closeModal();
  if (event.target === infoModal) closeInfoModal();
};

// Close search on Escape
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    const box = document.getElementById("searchBox");
    if (box.classList.contains("visible")) toggleSearch();
  }
});

window.onload = function () {
  initTheme();
  fetchAndRenderItems();
};
