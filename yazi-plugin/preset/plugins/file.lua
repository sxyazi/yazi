local M = {}

local function detect_os()
  local os_name
  if os.getenv('OS') then
    os_name = os.getenv('OS')
    if os_name:find('Windows') then
      return 'Windows'
    end
  end

  local uname_handle = io.popen('uname')
  if uname_handle then
    local uname_result = uname_handle:read('*a')
    uname_handle:close()
    if uname_result:find('Linux') then
      return 'Linux'
    elseif uname_result:find('Darwin') then
      return 'macOS'
    end
  end

  return 'Unknown'
end

function M:peek()
  local cmd = os.getenv('YAZI_FILE_ONE') or 'file'
  local options = detect_os() == 'Windows' and '-b' or '-bL'
  local output, code = Command(cmd):args({options, tostring(self.file.url)}):stdout(Command.PIPED):output()

  local p
  if output then
    p = ui.Paragraph.parse(self.area, '----- File Type Classification -----\n\n' .. output.stdout)
  else
    p =
      ui.Paragraph(
      self.area,
      {
        ui.Line(string.format('Spawn `%s` command returns %s', cmd, code))
      }
    )
  end

  ya.preview_widgets(self, {p:wrap(ui.Paragraph.WRAP)})
end

function M:seek()
end

return M
