vim.g.rustaceanvim = vim.tbl_deep_extend("force", vim.g.rustaceanvim or {}, {
  server = {
    default_settings = {
      ["rust-analyzer"] = {
        cfg = {
          setTest = false,
        },
        cargo = {
          allTargets = false,
        },
        check = {
          allTargets = false,
        },
      },
    },
  },
})
