import { invoke } from '@tauri-apps/api/tauri';

invoke('get_products')
  .then((res) => {
    const products = JSON.parse(res);
    console.log("Produk dari backend:", products);
    // Nanti render ke UI di sini
  })
  .catch((err) => {
    console.error("Gagal ambil produk:", err);
  });
