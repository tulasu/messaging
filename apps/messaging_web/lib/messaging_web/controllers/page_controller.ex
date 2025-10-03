defmodule MessagingWeb.PageController do
  use MessagingWeb, :controller

  def home(conn, _params) do
    render(conn, :home)
  end
end
